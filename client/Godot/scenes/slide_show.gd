extends Node3D

@export var dir: String
@export var slide_num: int
var page = 1

# Called when the node enters the scene tree for the first time.
func _ready():
	pass


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	$SubViewport/TextureRect.texture = ResourceLoader.load(dir+str(page)+".jpg")

func slide_select(num:int):
	page = num
