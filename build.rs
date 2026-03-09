fn main() {
    prost_build::Config::new()
        .compile_protos(&["proto/flights.proto"], &["proto/"])
        .expect("failed to compile protobuf");
}
