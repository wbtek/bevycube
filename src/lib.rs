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

pub mod roundel;
pub mod ui;
pub mod world;

use crate::roundel::*;

use bevy::asset::embedded_asset;
use bevy::image::*;
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct DiskParms {
    pub rotation_speed: f32,
}
#[derive(Debug, Resource)]
pub struct CubeParms {
    pub rotation_speed: f32,
}

#[derive(Debug, Resource, Default)]
pub struct EntityTable {
    pub cube: Option<Entity>,
    pub disk: Option<Entity>,
    pub ground: Option<Entity>,
    pub settings: Option<Entity>,
    pub set_anisotropic: Option<Entity>,
    pub set_mipmaps: Option<Entity>,
    pub set_resolution: Option<Entity>,
    pub set_fps: Option<Entity>,
    pub safety_disk: Option<Entity>,
    pub safety_disk_hidden: Option<Entity>,
    pub main_anchor: Option<Entity>,
    pub main_camera: Option<Entity>,
    pub ocean: Option<Entity>,
}

pub struct DemoAssetsPlugin;
impl Plugin for DemoAssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "media/wbtekbg2b512.jpg");
        embedded_asset!(app, "media/settings.jpg");
        embedded_asset!(app, "media/diamond_sprite.jpg");
    }
}

#[derive(Resource)]
pub struct OceanBuffer {
    pub current: Vec<f32>,
    pub previous: Vec<f32>,
    pub size: usize,
}

impl OceanBuffer {
    pub fn new(size: usize) -> Self {
        let count = size * size;
        Self {
            current: vec![0.0; count],
            previous: vec![0.0; count],
            size,
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
    }

    /// Injects a vertical displacement at a specific world coordinate.
    /// x, z: World coordinates (-10 to 10)
    /// magnitude: Height of the splash (e.g., 0.5)
    /// diameter: How many vertices are affected (e.g., 1.0)
    pub fn splash(&mut self, x: f32, z: f32, magnitude: f32, diameter: f32) {
        let size = self.size as f32;
        let spacing = 20.0 / (size - 1.0);
        let r_sq = (diameter / 2.0).powi(2);

        for row in 0..self.size {
            for col in 0..self.size {
                let i = row * self.size + col;
                let vx = (col as f32 * spacing) - 10.0;
                let vz = (row as f32 * spacing) - 10.0;

                let dist_sq = (vx - x).powi(2) + (vz - z).powi(2);
                if dist_sq < r_sq {
                    // We add to current so it doesn't "snap" back instantly
                    // A smooth falloff makes the splash look less "blocky"
                    let falloff = 1.0 - (dist_sq / r_sq).sqrt();
                    self.current[i] += magnitude * falloff;
                }
            }
        }
    }

    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        let size = self.size as f32;

        let col = ((x + 10.0) / 20.0 * (size - 1.0))
            .round()
            .clamp(0.0, size - 1.0) as usize;
        let row = ((z + 10.0) / 20.0 * (size - 1.0))
            .round()
            .clamp(0.0, size - 1.0) as usize;

        self.current[row * self.size + col]
    }
}
