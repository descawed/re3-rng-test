use std::collections::HashMap;

const SEED: u16 = 0x6CA4;
//const OFFSETS: [u8; 256] = [21, 103, 44, 192, 19, 175, 9, 249, 99, 189, 12, 128, 23, 69, 92, 112, 196, 86, 22, 11, 3, 218, 166, 46, 66, 140, 206, 30, 110, 116, 255, 89, 34, 208, 14, 180, 104, 35, 172, 238, 210, 65, 33, 222, 236, 194, 161, 29, 109, 36, 188, 126, 250, 147, 234, 16, 43, 64, 84, 38, 244, 133, 54, 96, 80, 246, 45, 144, 146, 162, 18, 28, 241, 131, 6, 68, 239, 142, 114, 91, 53, 199, 49, 191, 168, 134, 42, 17, 195, 5, 176, 39, 152, 71, 167, 151, 129, 98, 160, 61, 27, 25, 87, 154, 122, 221, 115, 15, 1, 117, 182, 31, 121, 138, 145, 169, 10, 186, 48, 149, 233, 203, 155, 102, 125, 47, 26, 78, 2, 235, 217, 135, 163, 62, 215, 229, 130, 171, 113, 170, 213, 156, 7, 137, 219, 81, 106, 181, 24, 57, 225, 173, 50, 214, 76, 150, 201, 90, 136, 247, 164, 41, 207, 55, 190, 159, 141, 123, 216, 232, 52, 158, 183, 177, 118, 40, 227, 230, 132, 198, 67, 82, 107, 95, 101, 252, 204, 56, 105, 4, 178, 139, 197, 120, 60, 223, 72, 212, 165, 228, 187, 193, 220, 74, 108, 251, 70, 179, 153, 143, 75, 205, 93, 111, 237, 59, 248, 226, 209, 224, 254, 200, 63, 124, 174, 58, 32, 211, 20, 37, 51, 100, 231, 245, 119, 185, 240, 97, 242, 0, 253, 243, 94, 127, 73, 184, 148, 79, 13, 77, 83, 85, 88, 8, 157, 202];

const fn re3_rng(n: u16) -> u16 {
    ((n << 1) & 0xff00) | ((n.overflowing_mul(258).0 >> 8) & 0x00ff)
}

const fn re3_rng2(n: u16) -> u16 {
    let v0 = (n >> 7) & 0xff;
    let v1 = n.overflowing_add(v0).0 & 0xff;
    v1 | (v0 << 8)
}

const SCRIPT_RNG: [u16; 24312] = const {
    let mut rng = SEED;
    let mut values = [0u16; 24312];
    let mut i = 0;
    while i < 24312 {
        let next_rng = re3_rng(rng);
        let hi = next_rng & 0xff;
        let lo = re3_rng(next_rng) & 0xff;
        values[i] = (hi << 8) | lo;
        rng = next_rng;
        i += 1;
    }

    // we should end up back where we started
    if rng != SEED {
        panic!("Unexpected RNG value");
    }

    values
};

const fn check_algo() {
    let mut rng = SEED;
    loop {
        let next_rng = re3_rng(rng);
        if re3_rng2(rng) != next_rng {
            panic!("Algorithms differ");
        }
        if next_rng == SEED {
            break;
        }
        rng = next_rng;
    }
}

fn check_rng(seed: u16) -> (u16, bool) {
    let mut found_freeze = false;
    let mut seen = [false; 0x8000];
    let mut byte_counts = [0; 0x100];
    let mut mod3 = [0, 0, 0];
    let mut mod4 = [0, 0, 0, 0];
    seen[seed as usize] = true;

    let mut rng = seed;
    let mut i = 0u16;
    loop {
        rng = re3_rng(rng) & 0x7fff;
        let index = rng as usize;
        let byte = index & 0xff;
        if seen[index] {
            break;
        } else if rng < 128 {
            println!("!!! Freeze point found at {} for seed {} on iteration {}", rng, seed, i);
            found_freeze = true;
        }
        seen[index] = true;
        byte_counts[byte] += 1;
        mod3[byte % 3] += 1;
        mod4[byte % 4] += 1;
        i += 1;
    }

    let num_values = seen.into_iter().filter(|&e| e).count();
    println!("Seed {} looped at value {} after {} iterations; {} unique values seen", seed, rng, i, num_values);
    for (byte, &count) in byte_counts.iter().enumerate() {
        println!("\t{:02X}: {}", byte, count);
    }

    let num_values = num_values as f32;
    println!("Bit counts:");
    for i in 0..8 {
        let bit = 1 << i;
        let mut total = 0;
        for (byte, &count) in byte_counts.iter().enumerate() {
            if (byte & bit) != 0 {
                total += count;
            }
        }

        println!("\t{:08b}: {} ({}%)", bit, total, (total as f32 / num_values) * 100.);
    }

    println!("Mod 3 probabilities:");
    for (i, count) in mod3.into_iter().enumerate() {
        println!("\t{i}: {count} ({}%)", (count as f32 / num_values) * 100.);
    }

    println!("Mod 4 probabilities:");
    for (i, count) in mod4.into_iter().enumerate() {
        println!("\t{i}: {count} ({}%)", (count as f32 / num_values) * 100.);
    }

    // probability that addition will carry into or borrow from the high 8 bits
    let mut total_probability = 0f32;
    for (byte, &count) in byte_counts.iter().enumerate() {
        let byte = byte as i16;
        let byte_probability = count as f32 / num_values; // 1./256.;
        total_probability += byte_probability * ((byte - 128).max(0) + (128 - byte).max(0)/*255 - byte*/) as f32 / 256.;
    }
    println!("Carry probability: {}%", total_probability * 100.);

    (i, found_freeze)
}

fn check_script_rng() {
    let mut script_rng_counts = HashMap::new();
    for &value in &SCRIPT_RNG {
        *script_rng_counts.entry(value).or_insert(0) += 1;
        for i in -128i16..128i16 {
            if i == 0 {
                continue; // we've already counted the value itself
            }

            let offset_value = value.overflowing_add_signed(i).0;
            *script_rng_counts.entry(offset_value).or_insert(0) += 1;
        }
    }

    let num_values: i32 = script_rng_counts.iter().map(|(_, &v)| v).sum();
    println!("{} unique script RNG values possible; {} total values", script_rng_counts.len(), num_values);

    let num_values = num_values as f32;
    println!("Bit counts:");
    for i in 0..16 {
        let bit = 1 << i;
        let mut total = 0;
        for (&rng, &count) in &script_rng_counts {
            if (rng & bit) != 0 {
                total += count;
            }
        }

        println!("\t{:016b}: {} ({}%)", bit, total, (total as f32 / num_values) * 100.);
    }

    let mut mod3 = [0, 0, 0];
    let mut pharm_probs = [0, 0, 0];
    for (&rng, &count) in &script_rng_counts {
        let remainder = (rng as usize) % 3;
        mod3[remainder] += count;
        // account for bug(?) in pharmacy computer logic
        if (rng & 0x8000) != 0 && remainder != 0 {
            pharm_probs[1] += count;
        } else {
            pharm_probs[remainder] += count;
        }
    }

    println!("Mod 3 probabilities:");
    for (i, count) in mod3.into_iter().enumerate() {
        println!("\t{i}: {count} ({}%)", (count as f32 / num_values) * 100.);
    }

    println!("Pharmacy password probabilities:");
    for (count, password) in pharm_probs.into_iter().zip(["Adravil", "Safsprin", "Aqua Cure"]) {
        println!("\t{password}: {count} ({}%)", (count as f32 / num_values) * 100.);
    }
}

fn main() {
    if re3_rng(SEED) != 0xd97d {
        panic!("RNG function is broken");
    }

    check_algo();
    check_rng(SEED);
    check_script_rng();

    /*check_rng(0x647d); // PDEMO00
    check_rng(0x22f9); // PDEMO01
    check_rng(0x175b); // PDEMO02*/

    /*let mut freeze_seeds = vec![];
    let mut loop_sizes = HashMap::new();
    for seed in 1..32768 {
        let (loop_size, found_freeze) = check_rng(seed);
        let seeds = loop_sizes.entry(loop_size).or_insert_with(Vec::new);
        seeds.push(seed);
        if found_freeze {
            freeze_seeds.push(seed);
        }
    }

    println!("\n\n\n{} seeds freeze: {:?}", freeze_seeds.len(), freeze_seeds);
    println!("Loop sizes:");
    for (loop_size, seeds) in loop_sizes {
        println!("\t{}: {} seeds", loop_size, seeds.len());
        if loop_size <= 1 {
            println!("\t\t{:?}", seeds);
        }
    }*/
}
