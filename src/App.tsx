import { Component } from 'react'
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import LabeledLinearProgress from './components/LabeledLinearProgress'
import SelectableList from './components/SelectableList'
import { Grid, MenuItem, Select } from '@mui/material'
import { Clip, Collection, InspectMode, Animation, ProgressPayload, Sprite } from './data/classes'
import { i18n, languages } from './data/i18n'

interface AppState {
  animationNames: string[]
  changedSprites: Sprite[]
  currentClip: Clip | null
  currentCollection: Collection | null
  currentCollections: Collection[]
  currentAnimation: Animation | null
  currentFrame: Sprite | null
  duplicateSprites: string[]
  inspectMode: InspectMode
  isPacking: boolean
  packProgress: number
}

export default class App extends Component<{}, AppState> {
  canvas: HTMLCanvasElement | null
  canvasContext: CanvasRenderingContext2D | null
  frameCache: HTMLImageElement[]
  framePaths: string[]
  frameIntervalID: number
  languages: string[]
  packIntervalID: number
  spritesPath: string

  constructor(props: {}) {
    super(props)
    this.state = {
      animationNames: [],
      changedSprites: [],
      currentClip: null,
      currentCollection: null,
      currentCollections: [],
      currentAnimation: null,
      currentFrame: null,
      duplicateSprites: [],
      inspectMode: InspectMode.Animation,
      isPacking: false,
      packProgress: 0,
    }

    this.canvas = null
    this.canvasContext = null
    this.frameCache = []
    this.frameIntervalID = -1
    this.framePaths = []
    this.languages = []
    this.packIntervalID = -1
    this.spritesPath = ""

    this.cancelPack = this.cancelPack.bind(this)
    this.checkForChangedSprites = this.checkForChangedSprites.bind(this)
    this.draw = this.draw.bind(this)
    this.findDuplicates = this.findDuplicates.bind(this)
    this.incrementFrameIndex = this.incrementFrameIndex.bind(this)
    this.packCollection = this.packCollection.bind(this)
    this.replaceDuplicates = this.replaceDuplicates.bind(this)
    this.setCurrentBackup = this.setCurrentBackup.bind(this)
    this.setCurrentClip = this.setCurrentClip.bind(this)
    this.setCurrentCollection = this.setCurrentCollection.bind(this)
    this.setCurrentAnimation = this.setCurrentAnimation.bind(this)
    this.setCurrentFrame = this.setCurrentFrame.bind(this)
    this.setCurrentSprite = this.setCurrentSprite.bind(this)
    this.setLanguage = this.setLanguage.bind(this)
    this.update = this.update.bind(this)
  }

  async componentDidMount() {
    await appWindow.listen("enablePack", (_) => {
      this.setState({ isPacking: false })
    })

    await appWindow.listen("progress", ({ event, payload }) => {
      this.setState({ packProgress: (payload as ProgressPayload).progress })
    })

    await appWindow.listen("refresh", (_) => {
      window.location.reload()
    })

    await invoke("get_sprites_path").then(path => this.spritesPath = path as string)
    await invoke("get_language").then(lang => {
      this.setLanguage(lang as string)
      for (const lang in languages) {
        this.languages.push(lang)
      }
    })
    await invoke("get_animation_list").then(animationList => {
      this.setState({ animationNames: animationList as string[] }, () => {
        if (this.state.animationNames.length > 0) {
          invoke("get_animation", { animationName: this.state.animationNames[0] })
            .then(animation => {
              const anim = animation as Animation
              this.setCurrentAnimation(anim.name)
            })
        }
      })
    })

    this.canvas = document.getElementById("clip-preview") as HTMLCanvasElement
    this.canvasContext = this.canvas?.getContext("2d") as CanvasRenderingContext2D

    window.requestAnimationFrame(this.update)
  }

  render() {
    return (
      <Grid container columns={{ xs: 7 }}>
        <Grid container item>
          <SelectableList items={this.state.animationNames}
            onSelectItem={this.setCurrentAnimation}
            selectedItem={this.state.currentAnimation?.name as string}
            title={i18n.animations} />
          <SelectableList items={this.state.currentAnimation?.clips.map(clip => clip.name) as string[]}
            onSelectItem={this.setCurrentClip}
            selectedItem={this.state.currentClip?.name as string}
            title={i18n.clips} />
          <SelectableList items={this.state.inspectMode == InspectMode.Collection
            ? this.state.currentCollection?.sprites.map(sprite => sprite.name) as string[]
            : this.state.currentClip?.frames.map(frame => frame.name) as string[]}
            onSelectItem={this.setCurrentFrame}
            selectedItem={this.state.currentFrame?.name as string}
            title={i18n.frames} />
          <Grid container item xs={2}>
            <Grid item>
              <canvas id="clip-preview" style={{ maxWidth: "100%", maxHeight: "100%" }} />
            </Grid>
            <SelectableList items={this.state.currentCollections?.map(cln => cln.name) as string[]}
              onSelectItem={this.setCurrentCollection}
              selectedItem={this.state.currentCollection?.name as string}
              title={i18n.atlases} />
          </Grid>
          <Grid container item xs={2}>
            <SelectableList items={this.state.duplicateSprites}
              onSelectItem={this.setCurrentBackup}
              selectedItem={this.state.currentFrame?.name as string}
              title={i18n.duplicates} />
            <Grid item>
              <button id="find-duplicates-button" onClick={this.findDuplicates}>
                {i18n.findDuplicates}
              </button>
            </Grid>
          </Grid>
          <Grid container item xs={2}>
            <SelectableList items={this.state.changedSprites.map(sprite => sprite.name) as string[]}
              onSelectItem={this.setCurrentSprite}
              selectedItem={this.state.currentFrame?.name as string}
              title={i18n.changedSprites} />
            <Grid item>
              <button id="restore-button" onClick={this.replaceDuplicates}>
                {i18n.replaceDuplicates}
              </button>
            </Grid>
          </Grid>
        </Grid>
        <Grid alignItems="stretch" container item xs={12}>
          <button hidden={this.state.isPacking || this.state.inspectMode != InspectMode.Collection}
            id="pack-button"
            style={{ padding: "16 16 8 8", width: "100%" }}
            onClick={this.packCollection}>
            {i18n.pack}
          </button>
          <Grid container item>
            <LabeledLinearProgress hidden={!this.state.isPacking}
              id="pack-progress-bar"
              text={`${i18n.packing}${this.state.currentCollection?.name as string}`}
              value={this.state.packProgress} />
            <Grid item xs={2}>
              <button hidden={!this.state.isPacking}
                id="cancel-pack-button"
                onClick={this.cancelPack}>
                {i18n.cancel}
              </button>
            </Grid>
          </Grid>
        </Grid>
        <Grid container item>
          <Select
            id="language-select"
            label={i18n.language}
            value={i18n.getLanguage()}>
            {this.languages.map(lang => {
              return <MenuItem className="language-item"
                key={lang}
                onClick={() => this.setLanguage(lang)}
                value={lang}>
                {lang}
              </MenuItem>
            })}
          </Select>
        </Grid>
      </Grid>
    )
  }

  cancelPack() {
    this.setState({ isPacking: false })
    invoke("cancel_pack")
  }

  checkForChangedSprites() {
    invoke("check_for_changed_sprites", { alreadyChangedSprites: this.state.changedSprites }).then(sprites => {
      var changedSprites = sprites as Sprite[]
      var stateChangedSprites = this.state.changedSprites
      for (const sprite of changedSprites) {
        if (!stateChangedSprites.some(s => s.name == sprite.name && s.id == sprite.id)) {
          stateChangedSprites.push(sprite)
        }
      }
      this.setState({ changedSprites: stateChangedSprites })
    })
  }

  debug(msg: string) {
    console.log("Debug: " + msg)
    invoke("debug", { msg })
  }

  draw() {
    if (this.canvas == null) {
      return
    }

    var img: HTMLImageElement | null = null
    if (this.state.inspectMode == InspectMode.Animation) {
      img = this.frameCache[this.state.currentClip?.currentFrameIndex as number]
    } else if (this.state.inspectMode == InspectMode.Collection) {
      img = this.frameCache[0]
    }

    if (img != null) {
      this.canvasContext?.clearRect(0, 0, this.canvas.width, this.canvas.height)
      this.canvasContext?.drawImage(img, 0, 0)
    }
  }

  findDuplicates() {
    invoke("find_duplicate_sprites").then(duplicates => {
      this.setState({ duplicateSprites: duplicates as string[] })
    })
  }

  incrementFrameIndex() {
    if (this.state.currentClip != null) {
      this.state.currentClip.currentFrameIndex++
      if (this.state.currentClip.currentFrameIndex >= this.state.currentClip.numFrames) {
        this.state.currentClip.currentFrameIndex = this.state.currentClip.loopStart
      }
      this.setState({ currentFrame: this.state.currentClip.frames[this.state.currentClip.currentFrameIndex] })
    }
  }

  packCollection() {
    this.setState({ packProgress: 0 })
    this.setState({ isPacking: true })
    invoke("pack_single_collection", { collectionName: this.state.currentCollection?.name as string })
  }

  replaceDuplicates() {
    invoke("replace_duplicate_sprites", { sourceSprite: this.state.currentFrame })
  }

  setCurrentBackup(backupName: string) {
    clearInterval(this.frameIntervalID as number)
    const frame = this.state.currentClip?.frames.find(frame => frame.name == backupName) as Sprite
    this.setState({ currentFrame: frame })
  }

  setCurrentClip(clipName: string) {
    clearInterval(this.frameIntervalID as number)
    const clip = this.state.currentAnimation?.clips.find(clip => clip.name == clipName)
    if (clip != undefined) {
      clip.currentFrameIndex = 0
      this.setState({ currentClip: clip, inspectMode: InspectMode.Animation })
      this.framePaths = clip.frames.map(frame => convertFileSrc(`${this.spritesPath}/${this.state.currentAnimation?.name}/${clip?.name}/${frame.name}`))
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

  setCurrentCollection(collectionName: string) {
    clearInterval(this.frameIntervalID)
    const collection = this.state.currentCollections?.find(cln => cln.name == collectionName) as Collection
    const img = new Image()
    img.onload = () => {
      if (this.canvas != null) {
        this.canvas.width = img.width
        this.canvas.height = img.height
      }
      this.frameCache = [img]
    }
    img.src = convertFileSrc(collection.path)
    this.setState({ currentCollection: collection, currentFrame: null, inspectMode: InspectMode.Collection })
  }

  setCurrentAnimation(animationName: string) {
    invoke("get_animation", { animationName })
      .then(animation => {
        const anim = animation as Animation
        this.setState({ currentAnimation: anim, inspectMode: InspectMode.Animation }, () => {
          if (anim.clips.length > 0) {
            this.setCurrentClip(anim.clips[0].name as string)
          }
        })
      })

    invoke("get_collections_from_animation_name", { animationName })
      .then(collections => {
        const clns = collections as Collection[]
        this.setState({ currentCollections: clns })
      })
  }

  setCurrentFrame(frameName: string) {
    clearInterval(this.frameIntervalID as number)
    if (this.state.inspectMode == InspectMode.Animation) {
      const frame = this.state.currentClip?.frames.find(frame => frame.name == frameName) as Sprite
      this.setState({ currentFrame: frame })
      const imgPath = convertFileSrc(`${this.spritesPath}/${frame.path}`)
      const img = new Image()
      img.onload = () => {
        if (this.state.currentClip != null) {
          this.state.currentClip.currentFrameIndex = 0
        }
        this.frameCache = [img]
        if (this.canvas != null) {
          this.canvas.width = img.width
          this.canvas.height = img.height
        }
      }
      img.src = imgPath
    } else if (this.state.inspectMode == InspectMode.Collection) {
      if (this.state.currentCollection != null) {
        const sprite = this.state.currentCollection.sprites.find(sprite => sprite.name == frameName) as Sprite
        const imgPath = convertFileSrc(`${this.spritesPath}/${sprite.path}`)
        const img = new Image()
        img.onload = () => {
          this.frameCache = [img]
          if (this.canvas != null) {
            this.canvas.width = img.width
            this.canvas.height = img.height
          }
        }
        img.src = imgPath
      }

      invoke("get_animation_name_from_collection_name", { collectionName: this.state.currentCollection?.name as string })
        .then(animationName => {
          invoke("get_animation", { animationName })
            .then(animation => {
              const anim = animation as Animation
              const clip = anim.clips.find(clip => clip.frames.find(frame => frame.name == frameName)) as Clip
              const frame = clip.frames.find(frame => frame.name == frameName) as Sprite
              this.setState({ currentClip: clip, currentFrame: frame })
            })
        })
    }
  }

  setCurrentSprite(spriteName: string) {
    invoke("get_collection_from_sprite_name", { spriteName }).then(collection => {
      invoke("debug", { msg: "Collection name: " + (collection as Collection).name })
      this.setState({ currentCollection: collection as Collection, inspectMode: InspectMode.Collection }, () => {
        this.setCurrentFrame(spriteName)
      })
    })
  }

  setLanguage(language: string) {
    i18n.setLanguage(language)
    invoke("set_language", { language: i18n.getLanguage(), menuItems: [i18n.quit, i18n.refresh, i18n.setSpritesPath] })
  }

  update() {
    this.checkForChangedSprites()
    this.draw()
    window.requestAnimationFrame(this.update)
  }
}