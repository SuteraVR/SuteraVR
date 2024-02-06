extends Node
class_name PlayerInstance

var Scene: Node3D
var PlayerId: int
var Clocker: ClockerConnection

const player_scene = preload("res://scenes/instance_player.tscn")


func _init(clocker: ClockerConnection, player_id: int):
	self.Clocker = clocker
	self.PlayerId = player_id
	self.Scene = player_scene.instantiate(PackedScene.GEN_EDIT_STATE_INSTANCE)
	self.Scene.visible = false
	add_child(self.Scene)
	print("Player %s initialized." % [PlayerId])

func appear():
	self.Scene.visible = true

func move(
	x: float, y: float, z: float,
	xx: float, xz: float, zx: float, zz: float,
):
	self.Scene.transform.origin = Vector3(x, y, z)
	self.Scene.transform.basis.x.x = xx
	self.Scene.transform.basis.x.z = xz
	self.Scene.transform.basis.z.x = zx
	self.Scene.transform.basis.z.z = zz
