extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	var clocker = get_parent()
	await(clocker.ready)
	clocker.connect_sutera_clocking_without_certverify(
		"localhost",
		"localhost:3501"
	)
	clocker.join_instance(1)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
