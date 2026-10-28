//! The graph view's layout, computed once at build time with Logseq's forces.
//!
//! Logseq lays its graph out in the browser with d3-force: a link force, a many-body
//! charge approximated with a Barnes–Hut quadtree, a collision force, a pull towards the
//! centre and a centring step, with a velocity decay of 0.5. This module runs the same
//! forces for d3's 300 ticks from d3's phyllotaxis start. It uses no random numbers, so
//! the same graph always gets the same positions. The charge step for each node only
//! reads the quadtree, so it runs on several threads without changing the result.
use std::fs;
use std::path::Path;

use crate::graph::ForceSettings;

/// Bump when the layout changes, so cached positions from older builds are not reused.
const LAYOUT_VERSION: u32 = 1;
/// d3's default: alpha falls from 1 to its minimum of 0.001 over 300 ticks.
const TICKS: usize = 300;
const ALPHA_MIN: f64 = 0.001;
const VELOCITY_KEEP: f64 = 0.5;
const THETA2: f64 = 0.25;
const COLLIDE_RADIUS: f64 = 26.0;
const CENTRE_PULL: f64 = 0.02;

/// Node positions for a graph of `count` nodes with these edges.
pub(super) fn layout(count: usize, edges: &[(u32, u32)], forces: ForceSettings) -> Vec<(f32, f32)> {
    let mut sim = Simulation::new(count, edges, forces);
    let decay = 1.0 - ALPHA_MIN.powf(1.0 / TICKS as f64);
    let mut alpha = 1.0;
    for _ in 0..TICKS {
        alpha += (0.0 - alpha) * decay;
        sim.tick(alpha);
    }
    sim.x
        .iter()
        .zip(&sim.y)
        .map(|(&x, &y)| (x as f32, y as f32))
        .collect()
}

/// [`layout`], reading and writing `cache` when given: positions are stored under a hash of
/// the titles, edges and forces. Returns the positions and whether the cache had them.
pub(super) fn cached_layout(
    cache: Option<&Path>,
    titles: &[&str],
    edges: &[(u32, u32)],
    forces: ForceSettings,
) -> (Vec<(f32, f32)>, bool) {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };
    feed(&LAYOUT_VERSION.to_le_bytes());
    for title in titles {
        feed(title.as_bytes());
        feed(&[0]);
    }
    for (a, b) in edges {
        feed(&a.to_le_bytes());
        feed(&b.to_le_bytes());
    }
    for value in [
        forces.link_distance,
        forces.charge_strength,
        forces.charge_range,
    ] {
        feed(&value.to_le_bytes());
    }
    let path = cache.map(|directory| directory.join(format!("graph-layout-{hash:016x}.bin")));
    if let Some(bytes) = path.as_ref().and_then(|path| fs::read(path).ok())
        && bytes.len() == titles.len() * 8
    {
        let positions = bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| {
                (
                    f32::from_le_bytes(chunk[..4].try_into().expect("4 bytes")),
                    f32::from_le_bytes(chunk[4..].try_into().expect("4 bytes")),
                )
            })
            .collect();
        return (positions, true);
    }
    let positions = layout(titles.len(), edges, forces);
    if let Some(path) = path {
        let bytes: Vec<u8> = positions
            .iter()
            .flat_map(|(x, y)| x.to_le_bytes().into_iter().chain(y.to_le_bytes()))
            .collect();
        // The cache only saves time; a failed write leaves the next build to lay out again.
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let partial = path.with_extension("partial");
        if fs::write(&partial, bytes).is_ok() {
            let _ = fs::rename(&partial, &path);
        }
    }
    (positions, false)
}

/// A tiny offset for coincident nodes, fixed by the pair so that runs agree.
fn jiggle(a: usize, b: usize) -> f64 {
    let mut hash =
        (a as u64).wrapping_mul(0x9e3779b97f4a7c15) ^ (b as u64).wrapping_add(0x632be5ab);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(0xbf58476d1ce4e5b9);
    ((hash >> 11) as f64 / (1_u64 << 53) as f64 - 0.5) * 1e-6
}

struct Simulation {
    x: Vec<f64>,
    y: Vec<f64>,
    vx: Vec<f64>,
    vy: Vec<f64>,
    edges: Vec<(usize, usize, f64, f64)>,
    distance: f64,
    strength: f64,
    distance_max2: f64,
}

/// A Barnes–Hut quadtree cell. Leaves hold a chain of coincident points.
struct Cell {
    x0: f64,
    y0: f64,
    size: f64,
    children: [u32; 4],
    point: u32,
    count: u32,
    cx: f64,
    cy: f64,
}

const NONE: u32 = u32::MAX;

struct Tree {
    cells: Vec<Cell>,
    next: Vec<u32>,
}

impl Tree {
    fn build(x: &[f64], y: &[f64]) -> Self {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (&px, &py) in x.iter().zip(y) {
            x0 = x0.min(px);
            y0 = y0.min(py);
            x1 = x1.max(px);
            y1 = y1.max(py);
        }
        let size = (x1 - x0).max(y1 - y0).max(1.0) * 1.0001;
        let mut tree = Self {
            cells: vec![Cell::new(x0, y0, size)],
            next: vec![NONE; x.len()],
        };
        for point in 0..x.len() {
            tree.insert(point as u32, x, y);
        }
        tree.accumulate(0, x, y);
        tree
    }

    fn insert(&mut self, point: u32, x: &[f64], y: &[f64]) {
        let (px, py) = (x[point as usize], y[point as usize]);
        let mut at = 0;
        loop {
            let cell = &self.cells[at];
            let leaf = cell.children == [NONE; 4];
            if leaf {
                if cell.point == NONE {
                    self.cells[at].point = point;
                    return;
                }
                let other = cell.point as usize;
                if (x[other] - px).abs() < 1e-9 && (y[other] - py).abs() < 1e-9 || cell.size < 1e-6
                {
                    self.next[point as usize] = cell.point;
                    self.cells[at].point = point;
                    return;
                }
                // Split: the leaf's chain moves into a child.
                let chain = cell.point;
                self.cells[at].point = NONE;
                let quadrant = self.quadrant(at, x[other], y[other]);
                let child = self.child(at, quadrant);
                self.cells[child].point = chain;
                continue;
            }
            let quadrant = self.quadrant(at, px, py);
            if self.cells[at].children[quadrant] == NONE {
                let child = self.child(at, quadrant);
                self.cells[child].point = point;
                return;
            }
            at = self.cells[at].children[quadrant] as usize;
        }
    }

    fn quadrant(&self, at: usize, px: f64, py: f64) -> usize {
        let cell = &self.cells[at];
        let half = cell.size / 2.0;
        usize::from(px >= cell.x0 + half) | (usize::from(py >= cell.y0 + half) << 1)
    }

    fn child(&mut self, at: usize, quadrant: usize) -> usize {
        let cell = &self.cells[at];
        let half = cell.size / 2.0;
        let child = Cell::new(
            cell.x0 + if quadrant & 1 == 1 { half } else { 0.0 },
            cell.y0 + if quadrant & 2 == 2 { half } else { 0.0 },
            half,
        );
        let index = self.cells.len();
        self.cells.push(child);
        self.cells[at].children[quadrant] = index as u32;
        index
    }

    /// Each cell's point count and centroid.
    fn accumulate(&mut self, at: usize, x: &[f64], y: &[f64]) {
        let mut stack = vec![(at, false)];
        while let Some((at, done)) = stack.pop() {
            let children = self.cells[at].children;
            if children == [NONE; 4] {
                let (mut count, mut point) = (0, self.cells[at].point);
                if point != NONE {
                    self.cells[at].cx = x[point as usize];
                    self.cells[at].cy = y[point as usize];
                }
                while point != NONE {
                    count += 1;
                    point = self.next[point as usize];
                }
                self.cells[at].count = count;
                continue;
            }
            if !done {
                stack.push((at, true));
                stack.extend(
                    children
                        .iter()
                        .filter(|&&c| c != NONE)
                        .map(|&c| (c as usize, false)),
                );
                continue;
            }
            let (mut count, mut sx, mut sy) = (0, 0.0, 0.0);
            for child in children.iter().filter(|&&c| c != NONE) {
                let child = &self.cells[*child as usize];
                count += child.count;
                sx += child.cx * f64::from(child.count);
                sy += child.cy * f64::from(child.count);
            }
            let cell = &mut self.cells[at];
            cell.count = count;
            if count > 0 {
                cell.cx = sx / f64::from(count);
                cell.cy = sy / f64::from(count);
            }
        }
    }
}

impl Cell {
    fn new(x0: f64, y0: f64, size: f64) -> Self {
        Self {
            x0,
            y0,
            size,
            children: [NONE; 4],
            point: NONE,
            count: 0,
            cx: 0.0,
            cy: 0.0,
        }
    }
}

impl Simulation {
    fn new(count: usize, edges: &[(u32, u32)], forces: ForceSettings) -> Self {
        // d3's phyllotaxis start.
        let angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        let (x, y): (Vec<f64>, Vec<f64>) = (0..count)
            .map(|i| {
                let radius = 10.0 * (0.5 + i as f64).sqrt();
                let turn = i as f64 * angle;
                (radius * turn.cos(), radius * turn.sin())
            })
            .unzip();
        let mut degree = vec![0_u32; count];
        for &(a, b) in edges {
            degree[a as usize] += 1;
            degree[b as usize] += 1;
        }
        let edges = edges
            .iter()
            .map(|&(a, b)| {
                let (da, db) = (f64::from(degree[a as usize]), f64::from(degree[b as usize]));
                (a as usize, b as usize, 1.0 / da.min(db), da / (da + db))
            })
            .collect();
        let range = f64::from(forces.charge_range.max(1));
        Self {
            vx: vec![0.0; count],
            vy: vec![0.0; count],
            x,
            y,
            edges,
            distance: f64::from(forces.link_distance),
            strength: f64::from(forces.charge_strength),
            distance_max2: range * range,
        }
    }

    fn tick(&mut self, alpha: f64) {
        self.link(alpha);
        self.charge(alpha);
        for _ in 0..2 {
            self.collide();
        }
        for i in 0..self.x.len() {
            self.vx[i] -= self.x[i] * CENTRE_PULL * alpha;
            self.vy[i] -= self.y[i] * CENTRE_PULL * alpha;
        }
        let n = self.x.len().max(1) as f64;
        let (mx, my) = (
            self.x.iter().sum::<f64>() / n,
            self.y.iter().sum::<f64>() / n,
        );
        for i in 0..self.x.len() {
            self.x[i] -= mx;
            self.y[i] -= my;
            self.vx[i] *= VELOCITY_KEEP;
            self.vy[i] *= VELOCITY_KEEP;
            self.x[i] += self.vx[i];
            self.y[i] += self.vy[i];
        }
    }

    /// d3's link force: a spring towards the link distance, weighted by the ends' degrees.
    fn link(&mut self, alpha: f64) {
        for &(source, target, strength, bias) in &self.edges {
            let mut dx = self.x[target] + self.vx[target] - self.x[source] - self.vx[source];
            let mut dy = self.y[target] + self.vy[target] - self.y[source] - self.vy[source];
            if dx == 0.0 {
                dx = jiggle(source, target);
            }
            if dy == 0.0 {
                dy = jiggle(target, source);
            }
            let length = (dx * dx + dy * dy).sqrt();
            let scale = (length - self.distance) / length * alpha * strength;
            dx *= scale;
            dy *= scale;
            self.vx[target] -= dx * bias;
            self.vy[target] -= dy * bias;
            self.vx[source] += dx * (1.0 - bias);
            self.vy[source] += dy * (1.0 - bias);
        }
    }

    /// d3's many-body force through a Barnes–Hut quadtree with theta 0.5.
    fn charge(&mut self, alpha: f64) {
        let tree = Tree::build(&self.x, &self.y);
        let count = self.x.len();
        let threads = std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 8);
        let chunk = count.div_ceil(threads).max(1);
        let (x, y) = (&self.x, &self.y);
        let (strength, max2) = (self.strength, self.distance_max2);
        let forces: Vec<(f64, f64)> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..count)
                .step_by(chunk)
                .map(|start| {
                    let tree = &tree;
                    scope.spawn(move || {
                        let mut stack = Vec::with_capacity(256);
                        (start..(start + chunk).min(count))
                            .map(|node| {
                                charge_on(tree, x, y, node, strength * alpha, max2, &mut stack)
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|worker| worker.join().expect("layout worker panicked"))
                .collect()
        });
        for (node, (fx, fy)) in forces.into_iter().enumerate() {
            self.vx[node] += fx;
            self.vy[node] += fy;
        }
    }

    /// d3's collision force for one radius, with a grid instead of a quadtree.
    fn collide(&mut self) {
        let radius = COLLIDE_RADIUS * 2.0;
        let count = self.x.len();
        let cell = |v: f64| (v / radius).floor() as i64;
        let mut keyed: Vec<((i64, i64), u32)> = (0..count)
            .map(|i| {
                (
                    (cell(self.x[i] + self.vx[i]), cell(self.y[i] + self.vy[i])),
                    i as u32,
                )
            })
            .collect();
        keyed.sort_unstable();
        let mut ranges = std::collections::HashMap::new();
        let mut start = 0;
        for i in 1..=keyed.len() {
            if i == keyed.len() || keyed[i].0 != keyed[start].0 {
                ranges.insert(keyed[start].0, (start, i));
                start = i;
            }
        }
        let homes: Vec<(i64, i64)> = {
            let mut homes = vec![(0, 0); count];
            for (key, node) in &keyed {
                homes[*node as usize] = *key;
            }
            homes
        };
        for (i, &(cx, cy)) in homes.iter().enumerate() {
            let xi = self.x[i] + self.vx[i];
            let yi = self.y[i] + self.vy[i];
            for gx in cx - 1..=cx + 1 {
                for gy in cy - 1..=cy + 1 {
                    let Some(&(from, to)) = ranges.get(&(gx, gy)) else {
                        continue;
                    };
                    for &(_, j) in &keyed[from..to] {
                        let j = j as usize;
                        if j <= i {
                            continue;
                        }
                        let mut dx = xi - self.x[j] - self.vx[j];
                        let mut dy = yi - self.y[j] - self.vy[j];
                        let mut l = dx * dx + dy * dy;
                        if l < radius * radius {
                            if dx == 0.0 {
                                dx = jiggle(i, j);
                                l += dx * dx;
                            }
                            if dy == 0.0 {
                                dy = jiggle(j, i);
                                l += dy * dy;
                            }
                            let length = l.sqrt();
                            let push = (radius - length) / length;
                            dx *= push;
                            dy *= push;
                            self.vx[i] += dx * 0.5;
                            self.vy[i] += dy * 0.5;
                            self.vx[j] -= dx * 0.5;
                            self.vy[j] -= dy * 0.5;
                        }
                    }
                }
            }
        }
    }
}

/// The charge on one node: d3's Barnes–Hut visit, skipping cells beyond the charge range.
fn charge_on(
    tree: &Tree,
    x: &[f64],
    y: &[f64],
    node: usize,
    strength: f64,
    max2: f64,
    stack: &mut Vec<usize>,
) -> (f64, f64) {
    let (px, py) = (x[node], y[node]);
    let (mut fx, mut fy) = (0.0, 0.0);
    stack.clear();
    stack.push(0);
    while let Some(at) = stack.pop() {
        let cell = &tree.cells[at];
        if cell.count == 0 {
            continue;
        }
        let mut dx = cell.cx - px;
        let mut dy = cell.cy - py;
        let mut l = dx * dx + dy * dy;
        let leaf = cell.children == [NONE; 4];
        if cell.size * cell.size / THETA2 < l {
            if l < max2 {
                if dx == 0.0 {
                    dx = jiggle(node, at);
                    l += dx * dx;
                }
                if dy == 0.0 {
                    dy = jiggle(at, node);
                    l += dy * dy;
                }
                if l < 1.0 {
                    l = l.sqrt();
                }
                let weight = strength * f64::from(cell.count) / l;
                fx += dx * weight;
                fy += dy * weight;
            }
            continue;
        }
        if !leaf {
            // No point in a cell farther than the charge range feels its charge.
            let nx = px.clamp(cell.x0, cell.x0 + cell.size) - px;
            let ny = py.clamp(cell.y0, cell.y0 + cell.size) - py;
            if nx * nx + ny * ny < max2 {
                stack.extend(
                    cell.children
                        .iter()
                        .filter(|&&c| c != NONE)
                        .map(|&c| c as usize),
                );
            }
            continue;
        }
        if l >= max2 {
            continue;
        }
        let mut point = cell.point;
        let alone = point as usize == node && tree.next[node] == NONE;
        if !alone {
            if dx == 0.0 {
                dx = jiggle(node, at);
                l += dx * dx;
            }
            if dy == 0.0 {
                dy = jiggle(at, node);
                l += dy * dy;
            }
            if l < 1.0 {
                l = l.sqrt();
            }
        }
        while point != NONE {
            if point as usize != node {
                let weight = strength / l;
                fx += dx * weight;
                fy += dy * weight;
            }
            point = tree.next[point as usize];
        }
    }
    (fx, fy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(count: u32) -> Vec<(u32, u32)> {
        (0..count)
            .map(|i| (i.min((i + 1) % count), i.max((i + 1) % count)))
            .collect()
    }

    #[test]
    fn layout_is_deterministic_and_spreads_nodes() {
        let edges = ring(40);
        let first = layout(60, &edges, ForceSettings::default());
        let second = layout(60, &edges, ForceSettings::default());
        assert_eq!(first, second);
        assert!(first.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
        // Collision keeps nodes apart; linked nodes stay near the link distance.
        let mut closest = f32::MAX;
        for (i, a) in first.iter().enumerate() {
            for b in &first[i + 1..] {
                closest = closest.min(((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt());
            }
        }
        assert!(closest > 20.0, "closest pair {closest}");
        let (a, b) = (first[0], first[1]);
        let link = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
        assert!((40.0..140.0).contains(&link), "link length {link}");
    }

    #[test]
    fn forces_change_the_layout() {
        let edges = ring(20);
        let near = layout(20, &edges, ForceSettings::default());
        let far = layout(
            20,
            &edges,
            ForceSettings {
                link_distance: 180,
                ..ForceSettings::default()
            },
        );
        let spread = |p: &[(f32, f32)]| p.iter().map(|(x, y)| x.abs() + y.abs()).sum::<f32>();
        assert!(spread(&far) > spread(&near));
    }

    #[test]
    fn the_cache_returns_the_same_positions() {
        let directory =
            std::env::temp_dir().join(format!("lsg-layout-cache-{}", std::process::id()));
        let titles = ["a", "b", "c"];
        let edges = [(0, 1), (1, 2)];
        let (first, hit) =
            cached_layout(Some(&directory), &titles, &edges, ForceSettings::default());
        assert!(!hit);
        let (second, hit) =
            cached_layout(Some(&directory), &titles, &edges, ForceSettings::default());
        assert!(hit);
        assert_eq!(first, second);
        let (_, hit) = cached_layout(
            Some(&directory),
            &titles,
            &[(0, 1)],
            ForceSettings::default(),
        );
        assert!(!hit, "a changed edge set misses the cache");
        let (_, hit) = cached_layout(None, &titles, &edges, ForceSettings::default());
        assert!(!hit);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
