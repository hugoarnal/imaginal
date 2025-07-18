pub fn display(matrix: Vec<bool>, width: u16, height: u16) {
    let mut line = String::new();

    for y in 0..height {
        line.clear();
        for x in 0..width {
            let i = x + y * width;
            let val = matrix.get(i as usize).unwrap();
            if *val {
                line.push_str("*");
            } else {
                line.push_str(".");
            }
        }
        println!("{}", line);

    }
}
