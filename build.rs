fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_service_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/primitives/proto/feed_outputs_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/primitives/proto/feed_outputs_service_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/regime/proto/feed_outputs_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/regime/proto/feed_outputs_service_v1.proto");

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

    primitives
        .compile_protos(
            &[
                "src/generated/primitives/proto/feed_outputs_v1.proto",
                "src/generated/primitives/proto/feed_outputs_service_v1.proto",
            ],
            &["src/generated/primitives/proto"],
        )
        .expect("compile primitives outputs protos");

    let mut regime = prost_build::Config::new();
    let regime_out_dir =
        std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("regime_outputs");
    std::fs::create_dir_all(&regime_out_dir).expect("create regime proto out dir");
    regime.out_dir(&regime_out_dir);
    regime.include_file("regime_outputs_proto.rs");

    regime
        .compile_protos(
            &[
                "src/generated/regime/proto/feed_outputs_v1.proto",
                "src/generated/regime/proto/feed_outputs_service_v1.proto",
            ],
            &["src/generated/regime/proto"],
        )
        .expect("compile regime outputs protos");
}
