extends CharacterBody3D

@onready var clocker: ClockerConnection = %Clocker
@onready var origin: XROrigin3D = $XROrigin3D;

func _ready():
	await clocker.ready

func _physics_process(_delta):
	clocker.report_player_transform(
		origin.transform.origin.x,
		origin.transform.origin.y,
		origin.transform.origin.z,
		origin.transform.basis.x.x,
		origin.transform.basis.x.z,
	)
