extends Node3D

@export var file: String

@onready var videoplayer: VideoStreamPlayer = $SubViewport/SubViewportContainer/VideoStreamPlayer
@onready var is_paused = false

func ready(delta):
	videoplayer.stream.file = file

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if Input.is_action_just_pressed("video_pause"):
		videoplayer.set_paused(!is_paused)
		is_paused = !is_paused
		print(!is_paused)
