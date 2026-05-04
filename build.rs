fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_service_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/primitives/proto/feed_outputs_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/primitives/proto/feed_outputs_service_v1.proto");

    let mut aggregator = prost_build::Config::new();
    aggregator.include_file("aggregator_bars_proto.rs");

    aggregator
        .compile_protos(
            &[
                "src/generated/aggregator/proto/feed_bars_v1.proto",
                "src/generated/aggregator/proto/feed_bars_service_v1.proto",
            ],
            &["src/generated/aggregator/proto"],
        )
        .expect("compile aggregator bars protos");

    let mut primitives = prost_build::Config::new();
    primitives.include_file("primitives_outputs_proto.rs");
    primitives.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    primitives
        .compile_protos(
            &[
                "src/generated/primitives/proto/feed_outputs_v1.proto",
                "src/generated/primitives/proto/feed_outputs_service_v1.proto",
            ],
            &["src/generated/primitives/proto"],
        )
        .expect("compile primitives outputs protos");
}
