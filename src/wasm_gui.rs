extern crate slime_seed_finder;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate palette;

#[cfg(feature = "wasm")]
use stdweb::js_export;
use palette::{Gradient, LinSrgb};

use slime_seed_finder::*;
use slime_seed_finder::slime::SlimeChunks;
use slime_seed_finder::biome_layers::Area;

#[cfg(feature = "wasm")]
fn main(){
    // Don't start, wait for user to press button
}

#[derive(Deserialize, Debug)]
pub struct Options {
    #[serde(default)]
    chunks: Vec<[i32; 2]>,
    #[serde(default)]
    no_chunks: Vec<[i32; 2]>,
}

js_deserializable!( Options );

#[cfg(feature = "wasm")]
#[js_export]
//pub fn slime_seed_finder(chunks_str: &str, no_chunks_str: &str) -> String {
//    let r = find_seed(chunks_str, no_chunks_str);
pub fn slime_seed_finder(o: Options) -> String {
    console!(log, "Hello from Rust");
    let r = find_seed(o);

    format!("Found {} seeds!\n{:#?}", r.len(), r)
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn extend48(s: &str) -> String {
    let x = match s.parse() {
        Ok(x) => {
            if x < (1u64 << 48) {
                x
            } else {
                let error_string = format!("Input must be lower than 2^48");
                console!(error, &error_string);
                return error_string;
            }
        }
        Err(e) => {
            let error_string = format!("{}", e);
            console!(error, &error_string);
            return error_string;
        }
    };

    let r = Rng::extend_long_48(x);
    let mut s = format!("Found {} seeds!\n", r.len());
    for seed in r {
        let seed = seed as i64;
        s.push_str(&format!("{}\n", seed));
    }

    s
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn count_candidates(o: Options) -> String {
    let c: Vec<_> = o.chunks.into_iter().map(|c| Chunk::new(c[0], c[1])).collect();
    let nc: Vec<_> = o.no_chunks.into_iter().map(|c| Chunk::new(c[0], c[1])).collect();

    if (c.len() == 0) && (nc.len() == 0) {
        return format!("{} * 2^30 candidates", 1 << 18);
    }
    let sc = SlimeChunks::new(&c, 0, &nc, 0);
    let num_cand = sc.num_low_18_candidates() as u32;
    return format!("{} * 2^30 candidates", num_cand);
}

pub fn find_seed(o: Options) -> Vec<u64> {
    let c: Vec<_> = o.chunks.into_iter().map(|c| Chunk::new(c[0], c[1])).collect();
    let nc: Vec<_> = o.no_chunks.into_iter().map(|c| Chunk::new(c[0], c[1])).collect();

    if (c.len() == 0) && (nc.len() == 0) {
        console!(log, "Can't find seed without chunks");
        return vec![];
    } 
    let sc = SlimeChunks::new(&c, 0, &nc, 0);
    let num_cand = sc.num_low_18_candidates() as u32;
    console!(log, format!("Found {} * 2^30 candidates", num_cand));
    console!(log, format!("ETA: about {} seconds", num_cand * 7));
    let seeds = sc.find_seed();

    {
        // Display only seeds that could be generated by java (empty box)
        let java_seeds: Vec<_> = seeds
            .iter()
            .map(|&s| Rng::extend_long_48(s))
            .collect();

        console!(log, format!("Java seeds: \n{:#?}", java_seeds));
    }

    seeds
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn generate_fragment(fx: i32, fy: i32, seed: String, frag_size: i32) -> Vec<u8> {
    let frag_size = frag_size as usize;
    let seed = if let Ok(s) = seed.parse() {
        s
    } else {
        console!(error, format!("{} is not a valid seed", seed));
        return vec![0; frag_size*frag_size*4];
    };

    let frag_size = frag_size as u64;
    let area = Area { x: fx as i64 * frag_size as i64, z: fy as i64 * frag_size as i64, w: frag_size, h: frag_size};
    //let last_layer = 43;
    //let map = cubiomes_test::call_layer(last_layer, seed, area);
    let v = biome_layers::generate_image(area, seed);

    v
}

pub fn slime_to_color(id: u32, total: u32, grad1: &Gradient<LinSrgb>) -> [u8; 4] {
    assert!(id <= total);
    // Gradient from red to green
    // http://blogs.perl.org/users/ovid/2010/12/perl101-red-to-green-gradient.html

    let num = id * 255 / total;
    let num = num as u8;
    let middle = 255 / 2;

    if id == 0 {
        // red
        [0xFF, 0x00, 0x00, 0xFF]
    } else if id == total {
        // white
        [0xFF, 0xFF, 0xFF, 0xFF]
    } else {
        let color = grad1.get(id as f32 / total as f32);
        [(color.red * 255.0) as u8, (color.green * 255.0) as u8, (color.blue * 255.0) as u8, 0xFF]
    }
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn generate_fragment_slime_map(fx: i32, fy: i32, seeds: Vec<String>, frag_size: usize) -> Vec<u8> {
    let seeds: Vec<u64> = seeds.into_iter().map(|s| s.parse().unwrap_or_else(|s| {
        console!(error, format!("{} is not a valid seed", s));
        panic!("{} is not a valid seed", s);
    })).collect();

    let frag_size = frag_size as u64;
    let area = Area { x: fx as i64 * frag_size as i64, z: fy as i64 * frag_size as i64, w: frag_size, h: frag_size};
    //let last_layer = 43;
    let num_seeds = seeds.len();
    if num_seeds > (0x10000) { // 65k seeds
        console!(log, "This may take a while");
    }
    let (w, h) = (area.w as usize, area.h as usize);
    let mut map_sum = vec![0; w*h];
    for seed in seeds {
        let map = slime::gen_map_from_seed(area, seed);
        for x in 0..w {
            for z in 0..h {
                let is_slime_chunk = map.a[(x, z)] != 0;
                if is_slime_chunk {
                    let i = z * h + x;
                    map_sum[i] += 1;
                }
            }
        }
    }

    let grad1 = Gradient::new(vec![
        LinSrgb::new(0.0, 0.0, 0.0),
        LinSrgb::new(1.0, 1.0, 0.0),
        LinSrgb::new(0.0, 1.0, 0.0),
    ]);
    let mut v = vec![0; w*h*4];
    for i in 0..w*h {
        let color = slime_to_color(map_sum[i], num_seeds as u32, &grad1);
        v[i*4+0] = color[0];
        v[i*4+1] = color[1];
        v[i*4+2] = color[2];
        v[i*4+3] = color[3];
    }

    v
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn add_2_48(seed: String) -> String {
    if let Ok(s) = seed.parse::<i64>() {
        format!("{}", s.wrapping_add(1 << 48))
    } else {
        seed
    }
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn sub_2_48(seed: String) -> String {
    if let Ok(s) = seed.parse::<i64>() {
        format!("{}", s.wrapping_sub(1 << 48))
    } else {
        seed
    }
}

#[cfg(feature = "wasm")]
#[js_export]
pub fn gen_test_seed_base_n_bits(base: String, n: String, bits: String) -> String {
    let base: i64 = base.parse().unwrap();
    let n: i64 = n.parse().unwrap();
    let bits: usize = bits.parse().unwrap();

    let sign = if n > 0 { 1 } else { -1 };
    let n = n * sign; //abs(n)

    let mut s = String::new();
    for i in 0..n {
        let x = base + i * sign * (1 << bits);
        s.push_str(&format!("{},\n", x));
    }

    s
}
