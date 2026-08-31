use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    #[cfg(windows)]
    {
        // Generate a valid app.ico file for the application
        generate_app_ico();

        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set("ProductName", "Apple Music Discord Presence");
        res.set("FileDescription", "Apple Music Discord Rich Presence");
        res.set("LegalCopyright", "Copyright (C) 2026");
        let _ = res.compile();
    }
}

fn generate_app_ico() {
    let path = Path::new("app.ico");
    if path.exists() {
        return;
    }

    // Create a 32x32 uncompressed 32-bit BMP (BGRA) icon
    let width: u32 = 32;
    let height: u32 = 32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let mut mask = vec![0u8; ((width * height) / 8) as usize];

    let center_x = 15.5;
    let center_y = 15.5;
    let radius = 14.5;
    let rad = -22.0f64.to_radians();
    let cos_t = rad.cos();
    let sin_t = rad.sin();

    for y in 0..height {
        for x in 0..width {
            // BMP icon stores rows from bottom to top
            let bmp_y = height - 1 - y;
            let idx = ((bmp_y * width + x) * 4) as usize;
            let mask_idx = ((bmp_y * width + x) / 8) as usize;
            let bit_idx = 7 - ((bmp_y * width + x) % 8);

            let fx = x as f64 + 0.5;
            let fy = y as f64 + 0.5;
            let dx = fx - center_x;
            let dy = fy - center_y;

            if dx * dx + dy * dy <= radius * radius {
                let n1_dx = fx - 11.0;
                let n1_dy = fy - 20.5;
                let r1_x = n1_dx * cos_t - n1_dy * sin_t;
                let r1_y = n1_dx * sin_t + n1_dy * cos_t;
                let in_head1 = (r1_x * r1_x) / (3.4 * 3.4) + (r1_y * r1_y) / (2.5 * 2.5) <= 1.0;

                let n2_dx = fx - 19.5;
                let n2_dy = fy - 17.5;
                let r2_x = n2_dx * cos_t - n2_dy * sin_t;
                let r2_y = n2_dx * sin_t + n2_dy * cos_t;
                let in_head2 = (r2_x * r2_x) / (3.4 * 3.4) + (r2_y * r2_y) / (2.5 * 2.5) <= 1.0;

                let in_stem1 = (12.0..=13.8).contains(&fx) && (8.5..=21.0).contains(&fy);
                let in_stem2 = (20.5..=22.3).contains(&fx) && (5.5..=18.0).contains(&fy);
                let beam_top = 8.5 + (fx - 12.0) * (-3.0 / 10.3);
                let in_beam = (12.0..=22.3).contains(&fx) && fy >= beam_top && fy <= beam_top + 2.9;

                if in_head1 || in_head2 || in_stem1 || in_stem2 || in_beam {
                    // Pure white (BGRA)
                    pixels[idx] = 255;
                    pixels[idx + 1] = 255;
                    pixels[idx + 2] = 255;
                    pixels[idx + 3] = 255;
                } else {
                    // Apple Music pink/red #FC3C44 (BGRA)
                    pixels[idx] = 0x44;
                    pixels[idx + 1] = 0x3C;
                    pixels[idx + 2] = 0xFC;
                    pixels[idx + 3] = 255;
                }
            } else {
                // Transparent
                mask[mask_idx] |= 1 << bit_idx;
            }
        }
    }

    let header_size = 40u32; // BITMAPINFOHEADER
    let image_size = width * height * 4;
    let mask_size = mask.len() as u32;
    let total_res_size = header_size + image_size + mask_size;

    let mut ico = Vec::new();
    // ICONDIR (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // idReserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // idType (1 = Icon)
    ico.extend_from_slice(&1u16.to_le_bytes()); // idCount (1 image)

    // ICONDIRENTRY (16 bytes)
    ico.push(32); // bWidth
    ico.push(32); // bHeight
    ico.push(0); // bColorCount
    ico.push(0); // bReserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // wPlanes
    ico.extend_from_slice(&32u16.to_le_bytes()); // wBitCount
    ico.extend_from_slice(&total_res_size.to_le_bytes()); // dwBytesInRes
    ico.extend_from_slice(&22u32.to_le_bytes()); // dwImageOffset (6 + 16 = 22)

    // BITMAPINFOHEADER (40 bytes)
    ico.extend_from_slice(&40u32.to_le_bytes()); // biSize
    ico.extend_from_slice(&32i32.to_le_bytes()); // biWidth
    ico.extend_from_slice(&(32i32 * 2).to_le_bytes()); // biHeight (double for icon AND+XOR)
    ico.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    ico.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    ico.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
    ico.extend_from_slice(&image_size.to_le_bytes()); // biSizeImage
    ico.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    ico.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    ico.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    ico.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // Color image data
    ico.extend_from_slice(&pixels);

    // AND mask data
    ico.extend_from_slice(&mask);

    if let Ok(mut f) = File::create("app.ico") {
        let _ = f.write_all(&ico);
    }
}
