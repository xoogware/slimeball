use slimeball_lib::SlimeWorld;

pub fn load_chunk_tops(world: SlimeWorld) {
    let mut top_blocks = vec!["".to_string(); 16 * 16];

    let chunk = &world.chunks[4];

    for (i, sec) in chunk.sections.iter().enumerate() {
        for block in 0..4096 {
            let v = sec.block_states.get(block);
            let coord = chunk_coord(i, block);

            if let Some(block_state) = v
                && block_state.name != "minecraft:air"
            {
                debug!("block index {block} ({:?}) in sec {i} val {v:?}", coord);
                top_blocks[block % 256] = block_state.name.clone();
            }
        }
    }

    debug!("{top_blocks:#?}");
}

fn chunk_coord(sec: usize, block: usize) -> (usize, usize, usize) {
    let (x, y, z) = index_to_coord(block);
    (x, y + (16 * sec), z)
}

fn index_to_coord(i: usize) -> (usize, usize, usize) {
    let layer_index = i % 256;
    (layer_index / 16, i / (16 * 16), layer_index % 16)
}
