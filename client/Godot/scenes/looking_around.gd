@tool
class_name LookingAround
extends XRToolsMovementProvider

## Movement provider order
@export var order : int = 30

const SPEED = 2
const TILT_LOWER_LIMIT := deg_to_rad(-90.0)
const TILT_UPPER_LIMIT := deg_to_rad(90.0)
const SENSITIVITY = 0.3

var mouse_input = false
var mouse_rotation: Vector2
var mouse_raw: Vector2

@onready var origin_node : XROrigin3D = XRHelpers.get_xr_origin(self)
@onready var camera_node : XRCamera3D = XRHelpers.get_xr_camera(self)


# Called when the node enters the scene tree for the first time.
func _ready():
	pass


func _unhandled_input(event):
	mouse_input = event is InputEventMouseMotion and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED
	if mouse_input:
		mouse_raw = Vector2(-event.relative.y, -event.relative.x)


func _update_camera(delta):
	mouse_rotation += mouse_raw * delta * SENSITIVITY
	mouse_rotation.x = clamp(mouse_rotation.x, TILT_LOWER_LIMIT, TILT_UPPER_LIMIT)
	mouse_rotation.y = fmod(mouse_rotation.y, 2*PI)
	camera_node.rotation.x = mouse_rotation.x
	origin_node.rotation.y = mouse_rotation.y
	camera_node.transform = camera_node.transform.orthonormalized()
	origin_node.transform = origin_node.transform.orthonormalized()
	mouse_raw = Vector2.ZERO


func _physics_process(delta):
	_update_camera(delta)
