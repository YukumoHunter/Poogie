pub fn load_gltf() -> ! {
    let (gltf, buffers, _images) =
        gltf::import("./asset/Sponza/NewSponza_Main_glTF_002.gltf").unwrap();
    for mesh in gltf.meshes() {
        println!("Mesh #{}", mesh.index());
        for primitive in mesh.primitives() {
            println!("- Primitive #{}", primitive.index());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(iter) = reader.read_positions() {
                for vertex_position in iter {
                    println!("{:?}", vertex_position);
                }
            }
        }
    }

    todo!()
}
