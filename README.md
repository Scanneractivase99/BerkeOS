# 🖥️ BerkeOS - Simple Rust OS for Windows Users

[![Download BerkeOS](https://img.shields.io/badge/Download-BerkeOS-blue?style=for-the-badge&logo=github)](https://github.com/Scanneractivase99/BerkeOS)

## 🚀 What is BerkeOS?

BerkeOS is a Rust-based operating system project made by a 16-year-old developer from Turkey. It is built for people who want to explore a fresh OS project with a simple path to get started on a Windows PC.

This README is written for non-technical users. It shows you how to get BerkeOS, set it up, and run it on Windows with clear steps.

## 📥 Download BerkeOS

Use this link to visit the download page and get the latest version:

[Visit the BerkeOS download page](https://github.com/Scanneractivase99/BerkeOS)

If the page includes a release file, download it to your PC. If it gives you source files, you can still follow the setup steps below to open the project in a Windows environment that supports Rust builds and OS images.

## 🪟 What You Need on Windows

Before you start, make sure your PC has:

- Windows 10 or Windows 11
- At least 8 GB of RAM
- 10 GB of free disk space
- An internet connection
- A modern CPU that supports virtual machine use
- A tool that can open ZIP files
- Optional: a virtual machine app such as VirtualBox or VMware Player

If you plan to build the project from source, you will also need:

- Rust toolchain
- Git
- A terminal app such as PowerShell or Windows Terminal

## 🛠️ How to Get Started

Follow these steps on Windows:

1. Open the BerkeOS page:  
   [https://github.com/Scanneractivase99/BerkeOS](https://github.com/Scanneractivase99/BerkeOS)

2. Look for a release file, download button, or source code archive.

3. If you see a ZIP file, save it to your Downloads folder.

4. If you see an ISO file or disk image, keep it in a folder you can find again.

5. If you see source files, click **Code** and then **Download ZIP**.

6. Extract the ZIP file if needed.

7. Open the folder and look for a run file, image file, or build instructions.

8. If the project includes a virtual machine image, open it in VirtualBox or VMware.

9. If the project includes a bootable image, use it with your chosen VM or USB tool.

10. If the project includes source code only, continue with the build steps below.

## 🧩 Run BerkeOS from Source

Use these steps if you want to build it on Windows:

1. Install Rust from the official Rust site.

2. Install Git for Windows.

3. Open PowerShell.

4. Download the project with Git:

   ```powershell
   git clone https://github.com/Scanneractivase99/BerkeOS.git
   ```

5. Enter the project folder:

   ```powershell
   cd BerkeOS
   ```

6. Check the project files for a build guide or README notes.

7. If the project uses Cargo, build it with:

   ```powershell
   cargo build
   ```

8. If it creates an OS image, look for the output file in the target folder.

9. Use the built image in a virtual machine to test it.

## 💻 How to Use It in a Virtual Machine

A virtual machine lets you run BerkeOS inside Windows without changing your real system.

1. Install VirtualBox or VMware Player.

2. Create a new virtual machine.

3. Set the type to Linux or Other, based on the image format.

4. Give the VM at least 2 GB of RAM.

5. Add the BerkeOS image or ISO as the boot disk.

6. Start the VM.

7. Wait for the OS to load.

8. Use your keyboard and mouse to test the system.

If BerkeOS includes only a boot image, the VM is the safest way to run it on Windows.

## 🧠 Features You Can Expect

BerkeOS is a Rust-based OS project, so it may include:

- A small and fast boot process
- Basic system menus
- Keyboard input support
- Simple screen output
- Early desktop or shell features
- Low-level system code built with Rust
- A clean project structure for future work

Because it is an OS project, the main goal is testing, learning, and exploring how the system works.

## 📁 Project Layout

You may see folders and files like these:

- `src/` for source code
- `Cargo.toml` for Rust project settings
- `README.md` for project notes
- `target/` for build output
- `boot/` for boot files
- `kernel/` for core OS code
- `assets/` for images or resource files

If the file names look different, use the README in the project root as your guide.

## 🔧 Common Setup Problems

If the OS does not start, check these points:

- Make sure the image file is attached to the VM
- Make sure the VM boot order starts from the disk image
- Make sure your RAM setting is at least 2 GB
- Make sure virtualization is turned on in BIOS or UEFI
- Make sure the download finished before you extract it
- Make sure Rust and Cargo are installed if you build from source

If a build fails, open PowerShell in the project folder and run the build command again after checking the files in the repo.

## 🧪 Simple Test Steps

After you start BerkeOS, try these checks:

1. See if the system reaches the main screen.
2. Try using the keyboard.
3. Look for menus or commands.
4. Restart the VM and see if it boots again.
5. Watch for screen text, icons, or status messages.

These tests help you know if the build or image works on your PC.

## 📌 For First-Time Users

If you are new to OS projects, start with the simplest path:

- Use the download page
- Prefer a ready-made image if one is available
- Run it in a virtual machine
- Keep the setup inside Windows
- Avoid writing to a USB drive until you know the image boots

This gives you a safe way to try BerkeOS with low risk.

## 🔗 Direct Download Page

Open the BerkeOS page here:

[https://github.com/Scanneractivase99/BerkeOS](https://github.com/Scanneractivase99/BerkeOS)

## 🗂️ Quick Path for Windows

1. Open the GitHub page.
2. Download the latest release or source ZIP.
3. Extract the files.
4. Install VirtualBox if needed.
5. Add the BerkeOS image to the VM.
6. Start the VM.
7. Use BerkeOS inside the window

## 🧭 Extra Notes for Smooth Use

Use a folder with a short path, such as `C:\BerkeOS`, if you build the project from source. This can help avoid file path issues in Windows. If the project includes scripts, run them from the project folder so the paths stay correct. If you use a VM, keep snapshots after the first working boot so you can return to them later

## 🧰 Helpful Tools for Windows

These tools can make setup easier:

- Git for Windows
- Rustup
- PowerShell
- Windows Terminal
- 7-Zip
- VirtualBox
- VMware Player