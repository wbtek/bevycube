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

pub const DISK_WORLD_Y: f32 = 0.0;
pub const DISK_WORLD_RADIUS: f32 = 4.0;
pub const DISK_WORLD_R2: f32 = DISK_WORLD_RADIUS * DISK_WORLD_RADIUS;

pub const CUBE_WORLD_SIDE_LEN: f32 = 2.25;

pub const GROUND_WORLD_Y: f32 = -3.0;
pub const GROUND_WORLD_SIDE_LEN: f32 = 28.0;

pub const OCEAN_WORLD_Y: f32 = -0.5;
pub const OCEAN_WORLD_SIDE_LEN: f32 = GROUND_WORLD_SIDE_LEN;
pub const OCEAN_TO_GROUND: f32 = GROUND_WORLD_Y - OCEAN_WORLD_Y;
