#!/bin/bash
set -e

if command -v dnf &> /dev/null; then
  echo "DNF based system detected. Dependencies are being loaded."
  
  sudo dnf install -y \
    cmake \
    ninja-build \
    gcc \
    gcc-c++ \
    vulkan-loader-devel \
    vulkan-headers \
    vulkan-validation-layers \
    glfw \
    glfw-devel \
    libshaderc-devel \
    wayland-protocols-devel

elif command -v apt-get &> /dev/null; then
  echo "APT based system detected. Dependencies are being loaded."
  
  sudo apt-get update
  
  sudo apt-get install -y \
    cmake \
    ninja \
    gcc \
    g++ \
    pkg-config \
    libvulkan-dev \
    vulkan-tools \
    libglfw3-dev \
    libshaderc-dev \
    wayland-protocols

elif command -v pacman &> /dev/null; then
  echo "Pacman based system detected. Dependencies are being loaded."
  
  sudo pacman -Sy --needed --noconfirm \
    cmake \
    ninja \
    gcc \
    vulkan-icd-loader \
    vulkan-headers \
    vulkan-validation-layers \
    glfw \
    shaderc \
    wayland-protocols

else
  echo "Error: This script currently only supports DNF, APT or Pacman package managers."
  echo "Please install the dependencies manually."
  exit 1
fi

echo "All system dependencies have been successfully installed!"
