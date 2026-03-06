//! # World Constants
//!
//! Defines world coordinates and dimensions for important objects.
//!
//! Top to bottom: Disk, Ocean, Ground.
//!
//! Ocean and Ground are same size but don't have to be.

// MIT License
//
// Copyright (c) 2026 - WBTek: Greg Slocum
// Division of WhiteBear Family, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// The Y-coordinate of the rotating disk in world space.
pub const DISK_WORLD_Y: f32 = 0.0;
/// The radius of the rotating disk in world units.
pub const DISK_WORLD_RADIUS: f32 = 4.0;
/// The squared radius of the disk, used for distance checks.
pub const DISK_WORLD_R2: f32 = DISK_WORLD_RADIUS * DISK_WORLD_RADIUS;

/// The side length of the rotating cube in world units.
pub const CUBE_WORLD_SIDE_LEN: f32 = 2.25;

/// The Y-coordinate of the ground plane in world space.
pub const GROUND_WORLD_Y: f32 = -3.0;
/// The side length of the ground plane in world units.
pub const GROUND_WORLD_SIDE_LEN: f32 = 28.0;

/// The Y-coordinate of the ocean surface in world space before waves.
pub const OCEAN_WORLD_Y: f32 = -0.5;
/// The side length of the ocean grid, matching the ground.
pub const OCEAN_WORLD_SIDE_LEN: f32 = GROUND_WORLD_SIDE_LEN;
/// The vertical distance between the ocean surface and the ground plane.
pub const OCEAN_TO_GROUND: f32 = GROUND_WORLD_Y - OCEAN_WORLD_Y;
