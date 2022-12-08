import { Component } from 'react'
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri'
import SelectableList from './components/SelectableList'
import Grid from '@mui/material/Grid'
import { Animation, Collection } from './data/classes'
import { getPath } from '@mui/system'

interface AppState {
  collectionNames: string[]
  currentAnimation: Animation | null
  currentCollection: Collection | null
  currentFrame: string | null
  framePaths: string[]
  intervalID: number | null
  spritesPath: string | null
}

export default class App extends Component<{}, AppState> {
  canvas: HTMLCanvasElement | null
  canvasContext: CanvasRenderingContext2D | null
  frameCache: HTMLImageElement[]
  framePaths: string[]
  intervalID: number
  spritesPath: string

  constructor(props: {}) {
    super(props)
    this.state = {
      collectionNames: [],
      currentAnimation: null,
      currentCollection: null,
      currentFrame: null,
      framePaths: [],
      intervalID: null,
      spritesPath: null,
    }

    this.canvas = null
    this.canvasContext = null
    this.frameCache = []
    this.framePaths = []
    this.intervalID = -1
    this.spritesPath = ""

    this.draw = this.draw.bind(this)
    this.incrementFrameIndex = this.incrementFrameIndex.bind(this)
    this.setCurrentAnimation = this.setCurrentAnimation.bind(this)
    this.setCurrentCollection = this.setCurrentCollection.bind(this)
    this.setCurrentFrame = this.setCurrentFrame.bind(this)
  }

  async componentDidMount() {
    await invoke('get_sprites_path').then(path => this.spritesPath = path as string)
    await invoke('get_collection_list').then(collectionList => {
      this.setState({ collectionNames: collectionList as string[] }, () => {
        if (this.state.collectionNames.length > 0) {
          invoke('get_collection', { collectionName: this.state.collectionNames[0] })
            .then(collection => {
              if (collection != null) {
                var cln = collection as Collection
                this.setCurrentCollection(cln.name)
              }
            })
        }
      })
    })

    this.canvas = document.getElementById("animation-preview") as HTMLCanvasElement
    this.canvasContext = this.canvas?.getContext("2d") as CanvasRenderingContext2D

    window.requestAnimationFrame(this.draw)
  }

  render() {
    return (
      <Grid container>
        <Grid container item>
          <Grid item xs={12}>
            {/* <img id="animation-preview"
              src={this.framePaths[this.currentAnimation?.currentFrameIndex as number]}
              alt="Animation Preview" /> */}
            <canvas id="animation-preview" />
          </Grid>
        </Grid>
        <Grid container item>
          <SelectableList items={this.state.collectionNames}
            onSelectItem={this.setCurrentCollection}
            selectedItem={this.state.currentCollection?.name as string}
            title="Collections" />
          <SelectableList items={this.state.currentCollection?.animations.map(anim => anim.name) as string[]}
            onSelectItem={this.setCurrentAnimation}
            selectedItem={this.state.currentAnimation?.name as string}
            title="Animations" />
          <SelectableList items={this.state.currentAnimation?.frames as string[]}
            onSelectItem={this.setCurrentFrame}
            selectedItem={this.state.currentFrame as string}
            title="Frames" />
        </Grid>
      </Grid>
    )
  }

  debug(msg: string) {
    console.log("Debug: " + msg)
    invoke('debug', { msg: msg })
  }

  draw() {
    const img = this.frameCache[this.state.currentAnimation?.currentFrameIndex as number]
    if (img != null) {
      this.canvasContext?.clearRect(0, 0, this.canvas?.width as number, this.canvas?.height as number)
      this.canvasContext?.drawImage(img, 0, 0)
    }

    window.requestAnimationFrame(this.draw)
  }

  incrementFrameIndex() {
    if (this.state.currentAnimation != null) {
      this.state.currentAnimation.currentFrameIndex++;
      if (this.state.currentAnimation.currentFrameIndex >= this.state.currentAnimation.numFrames) {
        this.state.currentAnimation.currentFrameIndex = this.state.currentAnimation.loopStart
      }
      this.setCurrentFrame(this.state.currentAnimation.frames[this.state.currentAnimation.currentFrameIndex] as string)
    }
  }

  setCurrentAnimation(animationName: string) {
    clearInterval(this.intervalID as number)
    var animation = this.state.currentCollection?.animations.find(anim => anim.name == animationName)
    if (animation != undefined) {
      animation.currentFrameIndex = 0;
      this.setState({ currentAnimation: animation })
      this.framePaths = animation.frames.map(frame => convertFileSrc(`${this.spritesPath}/${this.state.currentCollection?.name}/${animation?.name}/${frame}`))
      this.frameCache = []
      this.framePaths.forEach(path => {
        const img = new Image()
        img.onload = () => {
          this.frameCache.push(img)
        }
        img.src = path
      })
      this.setCurrentFrame(animation?.frames[0])
      this.intervalID = setInterval(this.incrementFrameIndex, 1000.0 / animation.fps)
    }
  }

  setCurrentCollection(collectionName: string) {
    invoke('get_collection', { collectionName: collectionName })
      .then(collection => {
        let cln = collection as Collection
        this.setState({ currentCollection: cln })
        if (cln.animations.length > 0) {
          this.setCurrentAnimation(cln.animations[0].name as string)
        }
      })
  }

  setCurrentFrame(frame: string) {
    this.setState({ currentFrame: frame })
  }
}