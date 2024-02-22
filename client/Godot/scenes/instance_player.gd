extends Node
class_name PlayerInstance

var Scene: Node3D
var PlayerId: int
var Clocker: ClockerConnection
var delay: Array[int] = [];

var ts: float = -1;
var te: float = -1;
var from: Transform3D;
var to: Transform3D;

const player_scene = preload("res://scenes/instance_player.tscn")
const player_scene1 = preload("res://tsukurun-world/avatars/ash/ash_1_0.tscn")
const player_scene2 = preload("res://tsukurun-world/avatars/ciel/ciel_1_0.tscn")
const player_scene3 = preload("res://scenes/3dmodels/Shapell.tscn")

func reset_delay():
	delay.clear()
	var start = Time.get_ticks_msec()
	for n in 10:
		delay.push_back(start + (n-10) * 16)

func _init(clocker: ClockerConnection, player_id: int):
	self.Clocker = clocker
	self.PlayerId = player_id
	self.Scene = player_scene.instantiate()
	self.Scene.visible = false
	add_child(self.Scene)
	print("Player %s initialized." % [PlayerId])
	reset_delay()


func appear():
	self.Scene.visible = true

func change_avatar(avatar_num:int):
	self.Scene.visible = false
	self.Scene.free()
	if(avatar_num==1):
		self.Scene = player_scene1.instantiate()
	elif(avatar_num==2):
		self.Scene = player_scene2.instantiate()
	elif(avatar_num==3):
		self.Scene = player_scene3.instantiate()
	else:
		print("error invalid value")
	self.Scene.visible = false
	add_child(self.Scene)
	self.Scene.visible = true

func calc_delay() -> int:
	var s = delay.pop_front()
	var e = Time.get_ticks_msec()
	if e - s > 1000:
		reset_delay()
		return 16
	delay.push_back(e)
	var sum: int = 0;
	for n in 9:
		sum += (delay[n+1] - delay[n])
	return sum / 9

func move(
	x: float, y: float, z: float,
	xx: float, xz: float, zx: float, zz: float,
):
	var delay = calc_delay()
	self.ts = Time.get_ticks_msec()
	self.te = self.ts + delay
	self.from = Transform3D(self.Scene.transform)
	self.to = Transform3D(self.Scene.transform)
	self.to.origin = Vector3(x, y, z)
	self.to.basis.x.x = xx
	self.to.basis.x.z = xz
	self.to.basis.z.x = zx
	self.to.basis.z.z = zz

func _process(_delta):
	if ts < 0:
		return;
	var ticks = Time.get_ticks_msec()
	if ticks > te:
		self.Scene.transform = self.to.orthonormalized()
		return
	var weight = (ticks - ts) / (te - ts)
	self.Scene.transform = self.from.interpolate_with(self.to, weight).orthonormalized()
	
