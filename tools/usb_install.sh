#!/bin/bash
# ============================================================
#  BerkeOS USB Installer
#  Supports both ISO hybrid mode and partition-based install
# ============================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Colors for menu
BOLD='\033[1m'

usage() {
    echo -e "${BOLD}BerkeOS USB Installer${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -d, --device <dev>    Target device (e.g., /dev/sdb)"
    echo "  -i, --iso <path>      Path to ISO (default: build/berkeos.iso)"
    echo "  -m, --mode <mode>     Installation mode:"
    echo "                          hybrid - Direct ISO write (recommended)"
    echo "                          partition - Partition-based install"
    echo "  -h, --help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                              # Interactive mode"
    echo "  $0 -d /dev/sdb -m hybrid        # Direct ISO write"
    echo "  $0 -d /dev/sdb -m partition    # Partition-based install"
    echo ""
    exit 0
}

# Parse arguments
DEVICE=""
ISO_PATH="build/berkeos.iso"
MODE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--device)
            DEVICE="$2"
            shift 2
            ;;
        -i|--iso)
            ISO_PATH="$2"
            shift 2
            ;;
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Check if running as root
if [ "$EUID" -ne 0 ] && [ -z "$DEVICE" ]; then 
    echo -e "${YELLOW}Note: Running as non-root. Some operations may require sudo.${NC}"
fi

# Find available USB devices
find_usb_devices() {
    echo -e "${CYAN}Available USB devices:${NC}"
    echo ""
    
    local i=1
    local devices=()
    
    for dev in /dev/sd?; do
        if [ -b "$dev" ]; then
            local size=$(lsblk -o SIZE -n --noheading 2>/dev/null | head -1 || echo "?")
            local vendor=$(cat /sys/block/${dev#/dev/}/device/vendor 2>/dev/null | tr -d ' ' || echo "Unknown")
            local model=$(cat /sys/block/${dev#/dev/}/device/model 2>/dev/null | tr -d ' ' || echo "Unknown")
            
            echo "  [$i] $dev (${vendor} ${model})"
            devices+=("$dev")
            ((i++))
        fi
    done
    
    if [ ${#devices[@]} -eq 0 ]; then
        echo "  No USB devices found. Plug in a USB drive and try again."
        return 1
    fi
    
    echo ""
    echo -n "Select device [1-${#devices[@]}]: "
    read -r choice
    
    if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le ${#devices[@]} ]; then
        DEVICE="${devices[$((choice-1))]}"
    else
        echo -e "${RED}Invalid selection!${NC}"
        exit 1
    fi
}

# ISO hybrid mode - direct write
install_hybrid() {
    local dev="$1"
    local iso="$2"
    
    echo -e "${GREEN}Mode: ISO Hybrid (Direct Write)${NC}"
    echo ""
    echo "This will write the ISO directly to the USB device."
    echo "Target: $dev"
    echo "Source: $iso"
    echo ""
    
    # Check if ISO exists
    if [ ! -f "$iso" ]; then
        echo -e "${RED}ISO not found: $iso${NC}"
        echo "Run ./build.sh first to create the ISO."
        exit 1
    fi
    
    # Unmount any mounted partitions
    echo -e "${CYAN}[1/4] Unmounting partitions...${NC}"
    for part in ${dev}*; do
        if mountpoint -q "$part" 2>/dev/null; then
            umount "$part" 2>/dev/null || sudo umount "$part" 2>/dev/null || true
        fi
    done
    
    # Write ISO to USB
    echo -e "${CYAN}[2/4] Writing ISO to USB...${NC}"
    echo -e "${YELLOW}This may take a few minutes...${NC}"
    
    if command -v pv &>/dev/null; then
        pv -p -b -r "$iso" | sudo dd of="$dev" bs=4M status=progress conv=notrunc
    else
        sudo dd if="$iso" of="$dev" bs=4M conv=notrunc status=progress
    fi
    
    # Sync
    echo -e "${CYAN}[3/4] Syncing...${NC}"
    sync
    
    # Set boot flag
    echo -e "${CYAN}[4/4] Setting up boot...${NC}"
    sudo parted -s "$dev" set 1 boot on 2>/dev/null || true
    
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo ""
    echo "Your USB is now bootable!"
    echo ""
    echo "To boot:"
    echo "  1. Plug in the USB drive"
    echo "  2. Select USB as boot device in BIOS/UEFI"
    echo "  3. Choose BerkeOS from the GRUB menu"
    echo ""
}

# Partition-based mode
install_partition() {
    local dev="$1"
    
    echo -e "${GREEN}Mode: Partition-Based Install${NC}"
    echo ""
    echo "This will create a partition and install GRUB."
    echo "Target: $dev"
    echo ""
    
    # Unmount
    echo -e "${CYAN}[1/6] Unmounting partitions...${NC}"
    for part in ${dev}*; do
        if mountpoint -q "$part" 2>/dev/null; then
            umount "$part" 2>/dev/null || sudo umount "$part" 2>/dev/null || true
        fi
    done
    
    # Create partition table
    echo -e "${CYAN}[2/6] Creating partition table...${NC}"
    sudo parted -s "$dev" mklabel msdos
    sudo parted -s "$dev" mkpart primary ext4 1MiB 100%
    sudo parted -s "$dev" set 1 boot on
    
    # Create filesystem
    echo -e "${CYAN}[3/6] Creating filesystem...${NC}"
    sudo mkfs.ext4 -F "${dev}1" >/dev/null 2>&1
    
    # Mount
    echo -e "${CYAN}[4/6] Mounting partition...${NC}"
    MOUNT_DIR=$(mktemp -d)
    sudo mount "${dev}1" "$MOUNT_DIR"
    
    # Copy kernel
    echo -e "${CYAN}[5/6] Installing BerkeOS...${NC}"
    sudo mkdir -p "$MOUNT_DIR/boot"
    sudo mkdir -p "$MOUNT_DIR/boot/grub"
    sudo cp build/berkeos.bin "$MOUNT_DIR/boot/"
    sudo cp build/isofiles/boot/grub/grub.cfg "$MOUNT_DIR/boot/grub/"
    
    # Install GRUB
    echo -e "${CYAN}[6/6] Installing GRUB...${NC}"
    sudo grub-install --target=i386-pc --boot-directory="$MOUNT_DIR" "$dev" 2>/dev/null || \
    sudo grub-install --target=x86_64-efi --boot-directory="$MOUNT_DIR" --efi-directory="$MOUNT_DIR" --removable 2>/dev/null || true
    
    # Cleanup
    sudo umount "$MOUNT_DIR"
    rmdir "$MOUNT_DIR"
    
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo ""
    echo "Your USB is now bootable!"
}

# Interactive mode
interactive() {
    echo -e "${BOLD}========================================${NC}"
    echo -e "${BOLD}     BerkeOS USB Installer${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""
    
    # Find device
    find_usb_devices
    
    echo ""
    echo -e "${CYAN}Select installation mode:${NC}"
    echo "  1. ISO Hybrid (recommended - direct write)"
    echo "  2. Partition-based (more flexible)"
    echo ""
    echo -n "Choice [1]: "
    read -r mode_choice
    
    case "$mode_choice" in
        2)
            MODE="partition"
            ;;
        *)
            MODE="hybrid"
            ;;
    esac
    
    echo ""
    read -p "Continue? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        echo "Aborted."
        exit 0
    fi
}

# Main
echo ""
if [ -z "$DEVICE" ]; then
    interactive
fi

if [ -z "$DEVICE" ]; then
    echo -e "${RED}Error: No device specified!${NC}"
    usage
fi

# Verify device
if [ ! -b "$DEVICE" ]; then
    echo -e "${RED}Error: $DEVICE is not a block device${NC}"
    exit 1
fi

# Check for dangerous devices
if [[ "$DEVICE" == "/dev/sda" ]] || [[ "$DEVICE" == "/dev/nvme0n1" ]]; then
    echo -e "${RED}ERROR: Refusing to write to $DEVICE (main drive)!${NC}"
    echo "Please specify a USB device (e.g., /dev/sdb)"
    exit 1
fi

# Run installation
case "$MODE" in
    partition)
        install_partition "$DEVICE"
        ;;
    *)
        install_hybrid "$DEVICE" "$ISO_PATH"
        ;;
esac
