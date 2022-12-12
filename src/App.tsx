import { Component } from 'react'
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import LabeledLinearProgress from './components/LabeledLinearProgress'
import SelectableList from './components/SelectableList'
import Grid from '@mui/material/Grid'
import { Clip, Library, ProgressPayload, Sprite } from './data/classes'

interface AppState {
  libraryNames: string[]
  currentClip: Clip | null
  currentLibrary: Library | null
  currentFrame: Sprite | null
  packProgress: number
}

export default class App extends Component<{}, AppState> {
  canvas: HTMLCanvasElement | null
  canvasContext: CanvasRenderingContext2D | null
  frameCache: HTMLImageElement[]
  framePaths: string[]
  frameIntervalID: number
  packIntervalID: number
  spritesPath: string

  constructor(props: {}) {
    super(props)
    this.state = {
      currentClip: null,
      currentLibrary: null,
      currentFrame: null,
      libraryNames: [],
      packProgress: 0,
    }

    this.canvas = null
    this.canvasContext = null
    this.frameCache = []
    this.frameIntervalID = -1
    this.framePaths = []
    this.packIntervalID = -1
    this.spritesPath = ""

    this.draw = this.draw.bind(this)
    this.incrementFrameIndex = this.incrementFrameIndex.bind(this)
    this.packLibrary = this.packLibrary.bind(this)
    this.setCurrentClip = this.setCurrentClip.bind(this)
    this.setCurrentLibrary = this.setCurrentLibrary.bind(this)
    this.setCurrentFrame = this.setCurrentFrame.bind(this)
  }

  async componentDidMount() {
    await appWindow.listen("progress", ({ event, payload }) => {
      this.setState({ packProgress: (payload as ProgressPayload).progress })
      if (this.state.packProgress >= 100) {
        var packButton = document.getElementById("pack-button") as HTMLButtonElement
        packButton.disabled = false
      }
    });

    await invoke('get_sprites_path').then(path => this.spritesPath = path as string)
    await invoke('get_library_list').then(libraryList => {
      this.setState({ libraryNames: libraryList as string[] }, () => {
        if (this.state.libraryNames.length > 0) {
          invoke('get_library', { libraryName: this.state.libraryNames[0] })
            .then(library => {
              if (library != null) {
                var lib = library as Library
                this.setCurrentLibrary(lib.name)
              }
            })
        }
      })
    })

    this.canvas = document.getElementById("clip-preview") as HTMLCanvasElement
    this.canvasContext = this.canvas?.getContext("2d") as CanvasRenderingContext2D

    window.requestAnimationFrame(this.draw)
  }

  render() {
    return (
      <Grid container>
        <Grid container item>
          <LabeledLinearProgress id="pack-progress-bar" value={this.state.packProgress} />
        </Grid>
        <Grid container item>
          <Grid item xs={12}>
            <canvas id="clip-preview" />
          </Grid>
        </Grid>
        <Grid container item>
          <SelectableList items={this.state.libraryNames}
            onSelectItem={this.setCurrentLibrary}
            selectedItem={this.state.currentLibrary?.name as string}
            title="Libraries" />
          <SelectableList items={this.state.currentLibrary?.clips.map(clip => clip.name) as string[]}
            onSelectItem={this.setCurrentClip}
            selectedItem={this.state.currentClip?.name as string}
            title="Clips" />
          <SelectableList items={this.state.currentClip?.frames.map(frame => frame.name) as string[]}
            onSelectItem={this.setCurrentFrame}
            selectedItem={this.state.currentFrame?.name as string}
            title="Frames" />
        </Grid>
        <Grid container item>
          <button id="pack-button" onClick={this.packLibrary}>Pack</button>
        </Grid>
      </Grid>
    )
  }

  debug(msg: string) {
    console.log("Debug: " + msg)
    invoke('debug', { msg: msg })
  }

  draw() { 
    const img = this.frameCache[this.state.currentClip?.currentFrameIndex as number]
    if (img != null) {
      this.canvasContext?.clearRect(0, 0, this.canvas?.width as number, this.canvas?.height as number)
      this.canvasContext?.drawImage(img, 0, 0)
    }

    window.requestAnimationFrame(this.draw)
  }

  incrementFrameIndex() {
    if (this.state.currentClip != null) {
      this.state.currentClip.currentFrameIndex++;
      if (this.state.currentClip.currentFrameIndex >= this.state.currentClip.numFrames) {
        this.state.currentClip.currentFrameIndex = this.state.currentClip.loopStart
      }
      this.setState({ currentFrame: this.state.currentClip.frames[this.state.currentClip.currentFrameIndex] })
    }
  }

  async packLibrary() {
    var packButton = document.getElementById("pack-button") as HTMLButtonElement
    packButton.disabled = true
    this.setState({ packProgress: 0 })

    invoke('pack_library', { libraryName: this.state.currentLibrary?.name as string, window: appWindow })
  }

  setCurrentClip(clipName: string) {
    clearInterval(this.frameIntervalID as number)
    var clip = this.state.currentLibrary?.clips.find(clip => clip.name == clipName)
    if (clip != undefined) {
      clip.currentFrameIndex = 0;
      this.setState({ currentClip: clip })
      this.framePaths = clip.frames.map(frame => convertFileSrc(`${this.spritesPath}/${this.state.currentLibrary?.name}/${clip?.name}/${frame.name}`))
      this.frameCache = []
      var maxWidth = 0
      var maxHeight = 0
      this.framePaths.forEach(path => {
        const img = new Image()
        img.onload = () => {
          if (img.width > maxWidth) {
            maxWidth = img.width
            if (this.canvas != null) {
              this.canvas.width = maxWidth
            }
          }
          if (img.height > maxHeight) {
            maxHeight = img.height
            if (this.canvas != null) {
              this.canvas.height = maxHeight
            }
          }
          this.frameCache.push(img)
        }
        img.src = path
      })
      this.setState({ currentFrame: clip?.frames[0] })
      this.frameIntervalID = setInterval(this.incrementFrameIndex, 1000.0 / clip.fps)
    }
  }

  setCurrentLibrary(libraryName: string) {
    invoke('get_library', { libraryName: libraryName })
      .then(library => {
        let lib = library as Library
        this.setState({ currentLibrary: lib })
        if (lib.clips.length > 0) {
          this.setCurrentClip(lib.clips[0].name as string)
        }
      })
  }

  setCurrentFrame(frameName: string) {
    clearInterval(this.frameIntervalID as number)
    let frame = this.state.currentClip?.frames.find(frame => frame.name == frameName) as Sprite
    this.setState({ currentFrame: frame })
    if (this.state.currentClip != null) {
      this.state.currentClip.currentFrameIndex = this.state.currentClip?.frames.indexOf(frame) as number
    }
  }
}