use std::io::Write;

pub fn write<W: std::io::Write>(
    pixels: &[[u8; 3]],
    width: usize,
    height: usize,
    output: W,
) -> std::io::Result<()> {
    let mut output = std::io::BufWriter::new(output);
    writeln!(output, "P3")?;
    writeln!(output, "{} {}", width, height)?;
    writeln!(output, "255")?;
    for pixel in pixels {
        writeln!(output, "{} {} {}", pixel[0], pixel[1], pixel[2])?;
    }
    Ok(())
}
