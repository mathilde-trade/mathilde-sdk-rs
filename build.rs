fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_v1.proto");
    println!("cargo:rerun-if-changed=src/generated/aggregator/proto/feed_bars_service_v1.proto");

    let mut config = prost_build::Config::new();
    config.include_file("aggregator_bars_proto.rs");

    config
        .compile_protos(
            &[
                "src/generated/aggregator/proto/feed_bars_v1.proto",
                "src/generated/aggregator/proto/feed_bars_service_v1.proto",
            ],
            &["src/generated/aggregator/proto"],
        )
        .expect("compile aggregator bars protos");
}
