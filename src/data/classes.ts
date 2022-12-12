export class Clip {
    currentFrameIndex: number
    currentTime: number
    duration: number
    fps: number
    frames: Sprite[]
    loopStart: number
    name: string
    numFrames: number

    constructor(frames: Sprite[], fps: number, loopStart: number, name: string) {
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

export class Library {
    name: string
    clips: Clip[]

    constructor(name: string, clips: Clip[]) {
        this.name = name
        this.clips = clips
    }
}

export class Sprite {
    id: number
    x: number
    y: number
    xr: number
    yr: number
    width: number
    height: number
    collectionName: string
    name: string
    path: string
    flipped: boolean

    constructor(
        id: number,
        x: number,
        y: number,
        xr: number,
        yr: number,
        width: number,
        height: number,
        collectionName: string,
        name: string,
        path: string,
        flipped: boolean
    ) {
        this.id = id;
        this.x = x;
        this.y = y;
        this.xr = xr;
        this.yr = yr;
        this.width = width;
        this.height = height;
        this.collectionName = collectionName
        this.name = name;
        this.path = path;
        this.flipped = flipped;
    }
}

export class ProgressPayload {
    progress: number;

    constructor(progress: number) {
        this.progress = progress
    }
}