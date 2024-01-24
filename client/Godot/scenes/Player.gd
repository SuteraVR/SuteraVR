extends CharacterBody3D

@export var SPEED = 2
var target_velocity = Vector3.ZERO

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _physics_process(delta):
	var direction = Vector3.ZERO
	
	direction.x = int(Input.is_action_pressed("move_right")) - int(Input.is_action_pressed("move_left"))
	direction.z = int(Input.is_action_pressed("move_back")) - int(Input.is_action_pressed("move_forward"))
		
	if direction != Vector3.ZERO:
		direction = direction.normalized()
		# $XROrigin3D.look_at(position + direction, Vector3.UP)
		
	target_velocity.x = direction.x * speed
	target_velocity.z = direction.z * speed
	
	velocity = target_velocity
	move_and_slide()
