fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/pmodel.proto", "proto/render.proto", "proto/save_notes.proto"], &["proto"])
        .unwrap();
}
