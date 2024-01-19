<template>
  <q-page id="home-page">
    <canvas id="clip-preview" width="100%" height="100%"></canvas>
    <div class="row">
      <div class="col">
        <SelectableList :items="app.animationNames" :title="animationsText" :selected-item="currentAnimationName"
          @select-item="setCurrentAnimation" />
        <SelectableList :items="clips" :title="clipsText" :selected-item="currentClipName"
          @select-item="setCurrentClip" />
        <SelectableList :items="frames" :title="framesText" :selected-item="currentFrameName"
          @select-item="setCurrentFrame" />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAppStore } from 'stores/app';

import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
import { appWindow } from '@tauri-apps/api/window';

import { Animation, Clip, Collection, InspectMode, PayloadProgress, Sprite } from 'src/data/classes';

import SelectableList from 'components/SelectableList.vue';

const { t, locale } = useI18n({ useScope: 'global' });
const app = useAppStore();

const canvas = ref<HTMLCanvasElement | null>(null);
const ctx = ref<CanvasRenderingContext2D | null>(null);

const frameIntervalId = ref<number | NodeJS.Timer>(-1);

const framePaths = ref<Array<string>>([]);
const spritesPath = ref<string>('');

const frameCache = ref<Array<HTMLImageElement>>([]);

onMounted(async (): Promise<void> => {
  await appWindow.listen('enablePack', () => {
    app.isPacking = false;
  });

  await appWindow.listen('progress', ({ payload }: { payload: PayloadProgress }) => {
    let progress = payload.progress;
    if (progress >= 100) {
      app.allowedToPack = false;
    }

    app.packProgress = progress;
  });

  await appWindow.listen('refresh', () => {
    window.location.reload();
  });

  await invoke('get_sprites_path').then((path: string) => spritesPath.value = path);
  await invoke('get_language').then((language: string) => {
    setLanguage(language);
    // for (const lang of languages) {
    //   this.languages.push(lang)
    // }
  });

  await invoke('get_mode').then((mode: string) => {
    app.mode = mode;
  });

  await invoke('get_animation_list').then((animationList: Array<string>) => {
    app.animationNames = animationList;
    if (app.animationNames.length > 0) {
      invoke('get_animation', { animationName: app.animationNames[0] })
        .then((animation: Animation) => {
          setCurrentAnimation(animation.name);
        })
    }
  });

  const canvasElement = document.getElementById('clip-preview');
  if (canvasElement) {
    canvas.value = canvasElement as HTMLCanvasElement;
    ctx.value = canvas.value.getContext('2d') as CanvasRenderingContext2D;
  }

  window.requestAnimationFrame(update);
});

const clips = computed((): Array<string> => {
  if (app.currentAnimation) {
    return app.currentAnimation.clips.map((clip: Clip): string => clip.name);
  }

  return [];
});

const frames = computed((): Array<string> => {
  if (app.currentClip) {
    return app.currentClip.frames.map((frame: Sprite): string => frame.name);
  }

  return [];
});

const animationsText = ref(t('animations'));
const clipsText = ref(t('clips'));
const framesText = ref(t('frames'));

const currentAnimationName = computed((): string => app.currentAnimation?.name ?? '');
const currentClipName = computed((): string => app.currentClip?.name ?? '');
const currentFrameName = computed((): string => app.currentFrame?.name ?? '');

const draw = (): void => {
  if (!canvas.value) {
    return;
  }

  var img: HTMLImageElement | null = null;
  if (app.inspectMode == InspectMode.Animation) {
    if (app.currentClip) {
      img = frameCache.value[app.currentClip.currentFrameIndex];
    }
  } else if (app.inspectMode == InspectMode.Collection) {
    img = frameCache.value[0];
  }

  if (img != null && ctx.value) {
    ctx.value.clearRect(0, 0, canvas.value.width, canvas.value.height);
    ctx.value.drawImage(img, 0, 0);
  }
};

const setLanguage = (language: string): void => {
  locale.value = language;
  invoke('set_language', { language: locale.value, menuItems: [t('quit'), t('refresh'), t('setSpritesPath')] });
};

const setCurrentBackup = (backupName: string): void => {
  clearInterval(frameIntervalId.value);
  if (app.currentClip) {
    const frame = app.currentClip.frames.find((frame: Sprite) => frame.name == backupName);
    if (frame) {
      app.currentFrame = frame;
    }
  }
}

app.currentFrame = null;

const setCurrentClip = (clipName: string): void => {
  clearInterval(frameIntervalId.value);
  const clip = app.currentAnimation?.clips.find(clip => clip.name == clipName);
  if (clip) {
    clip.currentFrameIndex = 0
    app.currentClip = clip;
    app.inspectMode = InspectMode.Animation;
    framePaths.value = clip.frames.map((frame: Sprite) => convertFileSrc(`${spritesPath.value}/${app.currentAnimation?.name}/${clip.name}/${frame.name}`))
    frameCache.value = [];
    var maxWidth = 0;
    var maxHeight = 0;
    for (const path of framePaths.value) {
      const img = new Image()
      img.onload = () => {
        if (img.width > maxWidth) {
          maxWidth = img.width
          if (canvas.value) {
            canvas.value.width = maxWidth
          }
        }
        if (img.height > maxHeight) {
          maxHeight = img.height
          if (canvas.value) {
            canvas.value.height = maxHeight
          }
        }
        +
          frameCache.value.push(img)
      }
      img.src = path
    }

    app.currentFrame = clip.frames[0];
    frameIntervalId.value = setInterval(app.incrementFrameIndex, 1000.0 / clip.fps)
  }
};

const setCurrentCollection = (collectionName: string): void => {
  clearInterval(frameIntervalId.value);
  const collection = app.currentCollections?.find(cln => cln.name == collectionName);
  if (collection) {
    const img = new Image()
    img.onload = () => {
      if (canvas.value) {
        canvas.value.width = img.width
        canvas.value.height = img.height
      }

      frameCache.value = [img]
    }
    img.src = convertFileSrc(collection.path)
    app.currentCollection = collection;
    app.currentFrame = null;
    app.inspectMode = InspectMode.Collection;
  }
};

const setCurrentAnimation = (animationName: string): void => {
  invoke('get_animation', { animationName })
    .then((animation: Animation) => {
      app.currentAnimation = animation
      app.inspectMode = InspectMode.Animation;
      if (animation.clips.length > 0) {
        setCurrentClip(animation.clips[0].name);
      }
    });

  invoke('get_collections_from_animation_name', { animationName })
    .then((collections: Array<Collection>) => {
      app.currentCollections = collections;
    });
};

const setCurrentFrame = (frameName: string): void => {
  clearInterval(frameIntervalId.value);
  if (app.inspectMode == InspectMode.Animation) {
    const frame = app.currentClip?.frames.find(frame => frame.name == frameName);
    if (frame) {
      app.currentFrame = frame;
      const imgPath = convertFileSrc(`${spritesPath.value}/${frame.path}`);
      const img = new Image();
      img.onload = () => {
        if (app.currentClip) {
          app.currentClip.currentFrameIndex = 0;
        }
        frameCache.value = [img];
        if (canvas.value) {
          canvas.value.width = img.width;
          canvas.value.height = img.height;
        }
      }

      img.src = imgPath;
    }
  } else if (app.inspectMode == InspectMode.Collection) {
    if (app.currentCollection) {
      const sprite = app.currentCollection.sprites.find(sprite => sprite.name == frameName);
      if (sprite) {
        const imgPath = convertFileSrc(`${spritesPath.value}/${sprite.path}`);
        const img = new Image();
        img.onload = () => {
          frameCache.value = [img];
          if (canvas.value) {
            canvas.value.width = img.width;
            canvas.value.height = img.height;
          }
        }

        img.src = imgPath;
      }
    }

    invoke('get_animation_name_from_collection_name', { collectionName: app.currentCollection?.name })
      .then((animationName: string) => {
        invoke('get_animation', { animationName })
          .then((animation: Animation) => {
            const clip = animation.clips.find(clip => clip.frames.find(frame => frame.name == frameName));
            if (clip) {
              app.currentClip = clip;
              const frame = clip.frames.find(frame => frame.name == frameName);
              if (frame) {
                app.currentFrame = frame;
              }
            }
          })
      })
  }
};

const update = (): void => {
  app.checkForChangedSprites();
  draw();

  window.requestAnimationFrame(update);
};
</script>
