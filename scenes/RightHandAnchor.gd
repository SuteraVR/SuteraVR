extends XRController3D


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	var This := get_tree().get_root().get_node("Node3D/XROrigin3D/RightHand")
	var ThisPos = This.get_global_position()
	var RightHandAnchor := get_tree().get_root().get_node("Node3D/XROrigin3D/PlayerBody/shapell").get_node("RightMarker3D")
	RightHandAnchor.set_global_position(ThisPos)
	var ThisRot = This.get_global_rotation()
	RightHandAnchor.set_global_rotation(ThisRot)
