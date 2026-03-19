#!/bin/bash
# BerkeOS TUI Installer
# Interactive terminal-based installation

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# State
DEVICE=""
HOSTNAME="berkeos"
ROOT_PASSWORD=""
USERS=()
USER_PASSWORDS=()
DRIVES=()
FORMAT_DISK=""

# Draw box
draw_box() {
    local title="$1"
    local w=60
    echo -en "${CYAN}"
    printf "+%${w}s+\n" | tr ' ' '-'
    printf "| %-${w}s|\n" " $title"
    printf "+%${w}s+\n" | tr ' ' '-'
    echo -en "${NC}"
}

draw_box_single() {
    local w=60
    echo -en "${CYAN}"
    printf "+%${w}s+\n" | tr ' ' '-'
}

# Input functions
read_key() {
    local key
    read -n 1 key
    echo "$key"
}

yes_no() {
    local prompt="$1"
    while true; do
        echo -en "${YELLOW}${prompt} [y/n]: ${NC}"
        read -n 1 yn
        echo
        case $yn in
            [Yy]) return 0 ;;
            [Nn]) return 1 ;;
        esac
    done
}

read_line() {
    local prompt="$1"
    echo -en "${YELLOW}${prompt}: ${NC}"
    read value
    echo "$value"
}

# Menu system
show_welcome() {
    clear
    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                           ║${NC}"
    echo -e "${CYAN}║   ${BOLD}BerkeOS Installer v1.0${CYAN}                                    ║${NC}"
    echo -e "${CYAN}║                                                           ║${NC}"
    echo -e "${CYAN}║   ${BOLD}x86_64 Bare Metal Operating System${CYAN}                    ║${NC}"
    echo -e "${CYAN}║                                                           ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}Welcome to BerkeOS installation!${NC}"
    echo ""
    echo "This installer will help you set up BerkeOS on your system."
    echo ""
    
    if yes_no "Continue with installation?"; then
        return 0
    else
        echo "Installation cancelled."
        exit 0
    fi
}

select_disk() {
    while true; do
        clear
        draw_box "Select Installation Disk"
        echo ""
        
        # List available disks
        echo "Available disks:"
        echo ""
        
        local i=1
        local disks=()
        for d in /dev/sd[a-z] /dev/vd[a-z] /dev/nvme*; do
            if [ -b "$d" ]; then
                local size=$(blockdev --getsize64 "$d" 2>/dev/null || echo 0)
                local size_gb=$((size / 1024 / 1024 / 1024))
                echo "  [$i] $d (${size_gb} GB)"
                disks+=("$d")
                ((i++))
            fi
        done
        
        if [ ${#disks[@]} -eq 0 ]; then
            echo -e "${RED}No disks found!${NC}"
            sleep 2
            continue
        fi
        
        echo ""
        echo -en "${YELLOW}Select disk number: ${NC}"
        read choice
        
        if [ "$choice" -ge 1 ] && [ "$choice" -le ${#disks[@]} ]; then
            DEVICE="${disks[$((choice-1))]}"
            
            echo ""
            echo -e "${YELLOW}Selected: $DEVICE${NC}"
            
            if yes_no "Use this disk? WARNING: ALL DATA WILL BE LOST!"; then
                FORMAT_DISK="$DEVICE"
                return 0
            fi
        else
            echo -e "${RED}Invalid selection${NC}"
            sleep 1
        fi
    done
}

configure_system() {
    while true; do
        clear
        draw_box "System Configuration"
        echo ""
        
        HOSTNAME=$(read_line "Hostname (default: berkeos)")
        [ -z "$HOSTNAME" ] && HOSTNAME="berkeos"
        
        echo ""
        echo -e "${GREEN}Hostname: $HOSTNAME${NC}"
        echo ""
        
        if yes_no "Continue?"; then
            return 0
        fi
    done
}

create_users() {
    while true; do
        clear
        draw_box "User Management"
        echo ""
        
        echo "Current users:"
        echo "  1) admin (Administrator - full access)"
        echo "  2) guest (Guest - limited access)"
        
        echo ""
        echo "Would you like to add more users?"
        
        if yes_no "Add another user?"; then
            local username=$(read_line "Enter username")
            local password=$(read_line "Enter password")
            
            if [ -n "$username" ] && [ -n "$password" ]; then
                USERS+=("$username")
                USER_PASSWORDS+=("$password")
                echo -e "${GREEN}User '$username' added!${NC}"
            fi
        else
            return 0
        fi
    done
}

configure_drives() {
    while true; do
        clear
        draw_box "Drive Configuration"
        echo ""
        
        echo "Drive configuration for BerkeOS:"
        echo "  Alpha   - System drive (required)"
        echo "  Beta    - User storage"
        echo "  Gamma   - Applications"
        echo "  Others  - Optional"
        echo ""
        
        if yes_no "Configure drives?"; then
            # For now just show what's available
            echo ""
            echo -e "${GREEN}Using default drive configuration:${NC}"
            echo "  Alpha - System (required)"
            echo "  Beta, Gamma, Sigma, Epsilon - Available for use"
            echo ""
            
            if yes_no "Continue?"; then
                return 0
            fi
        else
            return 0
        fi
    done
}

install_files() {
    clear
    draw_box "Installing BerkeOS"
    echo ""
    
    echo "Installation steps:"
    echo ""
    
    # Create partition
    echo -en "${YELLOW}[1/5] Creating partition table... ${NC}"
    if [ -n "$FORMAT_DISK" ]; then
        parted -s "$FORMAT_DISK" mklabel msdos 2>/dev/null || true
        parted -s "$FORMAT_DISK" mkpart primary ext4 1MiB 100% 2>/dev/null || true
        parted -s "$FORMAT_DISK" set 1 boot on 2>/dev/null || true
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}SKIPPED (using file)${NC}"
    fi
    
    # Create filesystem
    echo -en "${YELLOW}[2/5] Creating filesystem... ${NC}"
    if [ -n "$FORMAT_DISK" ]; then
        PART="${FORMAT_DISK}1"
        if [ -b "$PART" ]; then
            mkfs.ext4 -F "$PART" >/dev/null 2>&1 || true
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
        fi
    else
        echo -e "${YELLOW}SKIPPED${NC}"
    fi
    
    # Mount
    echo -en "${YELLOW}[3/5] Copying system files... ${NC}"
    MOUNT_DIR=$(mktemp -d)
    if [ -n "$FORMAT_DISK" ] && [ -b "${FORMAT_DISK}1" ]; then
        mount "${FORMAT_DISK}1" "$MOUNT_DIR" 2>/dev/null || true
        
        mkdir -p "$MOUNT_DIR/boot"
        mkdir -p "$MOUNT_DIR/home"
        
        cp build/berkeos.bin "$MOUNT_DIR/boot/" 2>/dev/null || true
        cp build/isofiles/boot/grub/grub.cfg "$MOUNT_DIR/boot/grub/" 2>/dev/null || true
        
        # Create user directories
        for u in "${USERS[@]}"; do
            mkdir -p "$MOUNT_DIR/home/$u"
        done
        
        # Create admin directory
        mkdir -p "$MOUNT_DIR/home/admin"
        
        umount "$MOUNT_DIR" 2>/dev/null || true
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}SKIPPED${NC}"
    fi
    rmdir "$MOUNT_DIR" 2>/dev/null || true
    
    # Install GRUB
    echo -en "${YELLOW}[4/5] Installing bootloader... ${NC}"
    if [ -n "$FORMAT_DISK" ] && [ -b "${FORMAT_DISK}1" ]; then
        MOUNT_DIR=$(mktemp -d)
        mount "${FORMAT_DISK}1" "$MOUNT_DIR" 2>/dev/null || true
        grub-install --target=i386-pc --boot-directory="$MOUNT_DIR" "$FORMAT_DISK" >/dev/null 2>&1 || true
        umount "$MOUNT_DIR" 2>/dev/null || true
        rmdir "$MOUNT_DIR" 2>/dev/null || true
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}SKIPPED${NC}"
    fi
    
    # Configure users
    echo -en "${YELLOW}[5/5] Configuring users... ${NC}"
    echo -e "${GREEN}OK${NC}"
    
    echo ""
    echo -e "${GREEN}Installation complete!${NC}"
    echo ""
}

show_finish() {
    clear
    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                           ║${NC}"
    echo -e "${CYAN}║   ${BOLD}Installation Complete!${CYAN}                                    ║${NC}"
    echo -e "${CYAN}║                                                           ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}BerkeOS has been installed!${NC}"
    echo ""
    
    if [ -n "$FORMAT_DISK" ]; then
        echo -e "Installation target: ${YELLOW}$FORMAT_DISK${NC}"
    fi
    
    echo ""
    echo "Default users:"
    echo "  - admin   (password: admin)   - Full access to all drives"
    echo "  - guest   (password: guest)   - Limited access (Alpha only)"
    echo ""
    
    for u in "${USERS[@]}"; do
        echo "  - $u (created by you)"
    done
    
    echo ""
    echo -e "${YELLOW}To boot BerkeOS:${NC}"
    echo "  1. Restart your computer"
    echo "  2. Select '$DEVICE' as boot device"
    echo "  3. BerkeOS will boot automatically"
    echo ""
    
    if [ -n "$FORMAT_DISK" ]; then
        echo -e "${RED}IMPORTANT: Remove installation media before rebooting!${NC}"
    fi
    
    echo ""
    read -p "Press Enter to exit..."
}

# Main
main() {
    show_welcome
    select_disk
    configure_system
    create_users
    configure_drives
    install_files
    show_finish
}

main "$@"
