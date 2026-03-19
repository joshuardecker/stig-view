use crate::Format;
use crate::detect_stig_format;

#[test]
fn test_saving_benchmark() {
    let format =
        detect_stig_format("../test_assets/packed.toml").expect("Could not load Xylok benchmark.");

    if let Format::Xylok(xylok_benchmark) = format {
        let benchmark = xylok_benchmark
            .convert()
            .expect("Could not convert benchmark.");
        benchmark.save().expect("Could not save benchmark.");
    } else {
        panic!("Incorrect format loaded.")
    }
}
