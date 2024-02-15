extends Node3D

@export var dir: String
var page = 1

# Called when the node enters the scene tree for the first time.
func _ready():
	pass


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	$SubViewport/TextureRect.texture = ResourceLoader.load("res://slides/rekut/"+str(page)+".jpg")
	if Input.is_action_just_pressed("slides_next"):
		page+=1
	if Input.is_action_just_pressed("slides_back"):
		page-=1
