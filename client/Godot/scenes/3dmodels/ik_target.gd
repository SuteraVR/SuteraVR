extends Marker3D

@export var step_propotional: float = 2
@export var adjacent_target: Node3D
@export var shapell_target: Node3D
@export var following_target:Node3D
@export var step_threshold: float = 0.8
@export var text = "a"
@export var is_stepping = false
var foot_origin
var distance
var now_global

func _init():
	foot_origin = global_position

func _process(delta):
	distance = following_target.global_position.distance_to(foot_origin)
	if is_stepping:
		if distance > step_threshold:
			foot_origin = following_target.global_position
			adjacent_target.is_stepping = true
			is_stepping = false
			print(text)
		global_position = following_target.global_position
		global_position.y = (-1)*step_propotional*distance*(distance - step_threshold)
	else:
		global_position = foot_origin
