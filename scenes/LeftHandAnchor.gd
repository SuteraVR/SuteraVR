extends XRController3D


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	var This := get_tree().get_root().get_node("Node3D/XROrigin3D/LeftHand")
	var ThisPos = This.get_global_position()
	var LeftHandAnchor := get_tree().get_root().get_node("Node3D/XROrigin3D/PlayerBody/shapell").get_node("LeftMarker3D")
	LeftHandAnchor.set_global_position(ThisPos)
	var ThisRot = This.get_global_rotation()
	LeftHandAnchor.set_global_rotation(ThisRot)
