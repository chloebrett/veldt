fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/load_sample.proto",
                "proto/pmodel.proto",
                "proto/putil.proto",
                "proto/render.proto",
                "proto/save_load.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
