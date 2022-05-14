#![cfg(test)]

pub const TEST_MAP: &str = r#"// Game: Fumohouse
// Format: Valve
// entity 0
{
"mapversion" "220"
"classname" "worldspawn"
"_tb_textures" "textures/map"
// brush 0
{
( -16 -64 -16 ) ( -16 -63 -16 ) ( -16 -64 -15 ) map/grass [ 0 -1 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -16 -16 ) ( -64 -16 -15 ) ( -63 -16 -16 ) map/black [ 1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( -64 -64 -16 ) ( -63 -64 -16 ) ( -64 -63 -16 ) map/wall [ -1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 64 16 ) ( 64 65 16 ) ( 65 64 16 ) map/accent [ 1 0 0 0 ] [ 0 -1 0 0 ] 0 1 1
( 64 16 16 ) ( 65 16 16 ) ( 64 16 17 ) map/dirt [ -1 0 0 0 ] [ 0 0 -1 0 ] 0 1 1
( 16 64 16 ) ( 16 64 17 ) ( 16 65 16 ) map/wall2 [ 0 1 0 0 ] [ 0 0 -1 0 ] 0 1 1
}
}"#;
