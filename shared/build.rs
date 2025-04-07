fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/load_sample.proto",
                "proto/pmodel.proto",
                "proto/render.proto",
                "proto/save_load.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
