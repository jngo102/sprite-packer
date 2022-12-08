export class Collection {
    name: string
    animations: Animation[]

    constructor(name: string, animations: Animation[]) {
        this.name = name
        this.animations = animations
    }
}

export class Animation {
    currentFrameIndex: number
    currentTime: number
    duration: number
    fps: number
    frames: string[]
    loopStart: number
    name: string
    numFrames: number

    constructor(frames: string[], fps: number, loopStart: number, name: string) {
        this.currentFrameIndex = 0
        this.currentTime = 0
        this.duration = frames.length * (1.0 / fps)
        this.frames = frames
        this.fps = fps
        this.loopStart = loopStart
        this.name = name
        this.numFrames = frames.length
    }
}