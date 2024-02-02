extends ClockerConnection


# Called when the node enters the scene tree for the first time.
func _ready():
	connect_sutera_clocking_without_certverify(
		"localhost",
		"localhost:3501"
	)
