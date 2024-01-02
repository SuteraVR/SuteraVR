use godot::engine::SkeletonIk3d;
use godot::prelude::*;
#[derive(GodotClass)]
#[class(init, tool, base=SkeletonIk3d)]
struct Shapell {
    #[base]
    skeletonik: Base<SkeletonIk3d>
}

use godot::engine::ISkeletonIk3d;

#[godot_api]
impl ISkeletonIk3d for Shapell {
    fn ready(&mut self){
        self.skeletonik.start();
    }
}
