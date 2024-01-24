extends CharacterBody3D

@export var SPEED = 2
var target_velocity = Vector3.ZERO



var mouse_input = false
var mouse_rotation: Vector3
var rotation_input: float
var tilt_input: float

# Called when the node enters the scene tree for the first time.
func _ready():
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	
func _input(event):
	if Input.is_action_just_pressed("release_cursor"):
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	if Input.is_action_just_pressed("left_click"):
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED

func _unhandled_input(event):
	mouse_input = event is InputEventMouseMotion and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED
	if mouse_input:
		rotation_input = -event.relative.x
		tilt_input = -event.relative.y
	print(Vector2(rotation_input, tilt_input))
	
func _update_camera(delta):
	var TILT_LOWER_LIMIT := deg_to_rad(-90.0)
	var TILT_UPPER_LIMIT := deg_to_rad(90.0)
	var SENSITIVITY = 0.3
	
	var CAMERA_CONTROLLER = $XROrigin3D/XRCamera3D
	
	mouse_rotation.x += tilt_input * delta * SENSITIVITY
	mouse_rotation.x = clamp(mouse_rotation.x, TILT_LOWER_LIMIT, TILT_UPPER_LIMIT)
	mouse_rotation.y += rotation_input * delta * SENSITIVITY
	
	CAMERA_CONTROLLER.transform.basis = Basis.from_euler(mouse_rotation)
	CAMERA_CONTROLLER.rotation.z = 0.0
	
	rotation_input = 0.0
	tilt_input = 0.0

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _physics_process(delta):
	_update_camera(delta)
	
	var direction = Vector3.ZERO
	
	direction.x = int(Input.is_action_pressed("move_right")) - int(Input.is_action_pressed("move_left"))
	direction.z = int(Input.is_action_pressed("move_back")) - int(Input.is_action_pressed("move_forward"))
		
	if direction != Vector3.ZERO:
		direction = direction.normalized()
		# $XROrigin3D.look_at(position + direction, Vector3.UP)
		
	target_velocity.x = direction.x * SPEED
	target_velocity.z = direction.z * SPEED
	
	velocity = target_velocity
	move_and_slide()
