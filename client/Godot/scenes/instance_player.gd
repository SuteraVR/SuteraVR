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
	add_child(self.Scene)
	print("Player %s initialized." % [PlayerId])
