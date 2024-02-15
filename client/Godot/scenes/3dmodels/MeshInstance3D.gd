extends MeshInstance3D
@export var mark:Marker3D
func _process(delta):
	position=mark.foot_origin
