extends Node
class_name PlayerInstance

var Scene: Node3D
var PlayerId: int
var Clocker: ClockerConnection

const player_scene = preload("res://scenes/instance_player.tscn")
const player_scene1 = preload("res://tsukurun-world/avatars/ash/ash_1_0.tscn")
const player_scene2 = preload("res://tsukurun-world/avatars/ciel/ciel_1_0.tscn")
const player_scene3 = preload("res://scenes/3dmodels/Shapell.tscn")


func _init(clocker: ClockerConnection, player_id: int):
	self.Clocker = clocker
	self.PlayerId = player_id
	self.Scene = player_scene.instantiate()
	self.Scene.visible = false
	add_child(self.Scene)
	print("Player %s initialized." % [PlayerId])

func appear():
	self.Scene.visible = true

func change_avatar(avatar_num:int):
	if(avatar_num==1):
		self.Scene = player_scene1.instantiate(PackedScene.GEN_EDIT_STATE_INSTANCE)
	elif(avatar_num==2):
		self.Scene = player_scene2.instantiate(PackedScene.GEN_EDIT_STATE_INSTANCE)
	elif(avatar_num==3):
		self.Scene = player_scene3.instantiate(PackedScene.GEN_EDIT_STATE_INSTANCE)
	else:
		print("error invalid value")

func move(
	x: float, y: float, z: float,
	xx: float, xz: float, zx: float, zz: float,
):
	self.Scene.transform.origin = Vector3(x, y, z)
	self.Scene.transform.basis.x.x = xx
	self.Scene.transform.basis.x.z = xz
	self.Scene.transform.basis.z.x = zx
	self.Scene.transform.basis.z.z = zz
