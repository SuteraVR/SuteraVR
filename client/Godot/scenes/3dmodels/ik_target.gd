extends Marker3D

@export var step_propotional: float = 0.5
@export var adjacent_target: Node3D
@export var step_threshold: float = 1.0

var is_stepping = true
var foot_origin
var distance

func _init():
	foot_origin = global_position

func _process(delta):
	if !adjacent_target.is_stepping:
		is_stepping = true
		distance = global_position.distance_to(foot_origin)
		if(distance > step_threshold):
			foot_origin = global_position
			foot_origin.y = 0
			print("true")
			is_stepping = false
		position.y = step_propotional*distance*distance
	
