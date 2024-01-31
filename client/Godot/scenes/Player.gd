extends CharacterBody3D

const SPEED = 2
const TILT_LOWER_LIMIT := deg_to_rad(-90.0)
const TILT_UPPER_LIMIT := deg_to_rad(90.0)
const SENSITIVITY = 0.3


var CAMERA_CONTROLLER: XRCamera3D

var mouse_input = false
var mouse_rotation: Vector2
var mouse_raw: Vector2

# Called when the node enters the scene tree for the first time.
func _ready():
	CAMERA_CONTROLLER = $XROrigin3D/XRCamera3D


func _unhandled_input(event):
	mouse_input = event is InputEventMouseMotion and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED
	if mouse_input:
		mouse_raw = Vector2(-event.relative.y, -event.relative.x)


func _update_camera(delta):
	mouse_rotation += mouse_raw * delta * SENSITIVITY
	mouse_rotation.x = clamp(mouse_rotation.x, TILT_LOWER_LIMIT, TILT_UPPER_LIMIT)
	mouse_rotation.y = fmod(mouse_rotation.y, 2*PI)
	
	CAMERA_CONTROLLER.transform.basis = Basis.from_euler(
		Vector3(mouse_rotation.x, mouse_rotation.y, 0)
	)
	CAMERA_CONTROLLER.rotation.z = 0.0
	
	mouse_raw = Vector2.ZERO


func _physics_process(delta):
	_update_camera(delta)
