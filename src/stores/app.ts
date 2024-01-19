import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/tauri';
import packageJson from '../../package.json';
import {
  Animation,
  Clip,
  Collection,
  InspectMode,
  Sprite,
  Theme,
} from 'src/data/classes';
import { Ref, ref } from 'vue';

interface AppState {
  version: string;
  allowedToPack: Ref<boolean>;
  animationNames: Ref<Array<string>>;
  changedSprites: Ref<Array<Sprite>>;
  currentAnimation: Ref<Animation | null>;
  currentClip: Ref<Clip | null>;
  currentCollection: Ref<Collection | null>;
  currentCollections: Ref<Array<Collection>>;
  currentFrame: Ref<Sprite | null>;
  mode: Ref<string>;
  duplicateSprites: Ref<Array<string>>;
  inspectMode: Ref<InspectMode>;
  isPacking: Ref<boolean>;
  packProgress: Ref<number>;
  theme: Ref<Theme>;

  cancelPack: () => void;
  changeMode: () => void;
  checkForChangedSprites: () => void;
  check: () => void;
  incrementFrameIndex: () => void;
  packCollections: () => void;
  replaceDuplicates: () => void;
  setCurrentSprite: (spriteName: string) => void;
  setLanguage(language: string): void;
  setMode(mode: string): void;
}

export const useAppStore = defineStore('app', (): AppState => {
  const version = packageJson.version;

  const allowedToPack = ref<boolean>(false);

  const animationNames = ref<Array<string>>([]);

  const changedSprites = ref<Array<Sprite>>([]);

  const currentAnimation = ref<Animation | null>(null);

  const currentClip = ref<Clip | null>(null);

  const currentCollection = ref<Collection | null>(null);

  const currentCollections = ref<Array<Collection>>([]);

  const currentFrame = ref<Sprite | null>(null);

  const mode = ref('dark');

  const duplicateSprites = ref<Array<string>>([]);

  const inspectMode = ref(InspectMode.Animation);

  const isPacking = ref<boolean>(false);

  const packProgress = ref<number>(0);

  const theme = ref(Theme.Dark);

  const cancelPack = (): void => {
    isPacking.value = false;
    invoke('cancel_pack');
  };

  const changeMode = (): void => {
    mode.value = 'dark';
  };

  const checkForChangedSprites = (): void => {
    invoke('check_for_changed_sprites', {
      alreadyChangedSprites: changedSprites.value,
    }).then((sprites: Array<Sprite>) => {
      for (const sprite of sprites) {
        if (
          !changedSprites.value.some(
            (s) => s.name == sprite.name && s.id == sprite.id
          )
        ) {
          changedSprites.value.push(sprite);
        }
      }
    });
  };

  const check = (): void => {
    invoke('check').then((problemSprites: Array<Sprite>) => {
      allowedToPack.value = problemSprites.length <= 0;
      for (const sprite of problemSprites) {
        if (!changedSprites.value.some((s) => s.name == sprite.name)) {
          changedSprites.value.push(sprite);
        }
      }
    });
  };

  const incrementFrameIndex = (): void => {
    if (currentFrame.value && currentClip.value) {
      currentClip.value.currentFrameIndex++;
      if (
        currentClip.value.currentFrameIndex >= currentClip.value?.frames.length
      ) {
        currentClip.value.currentFrameIndex = 0;
      }
    }
  };

  const packCollections = (): void => {
    packProgress.value = 0;
    isPacking.value = true;
    invoke('pack_single_collection', {
      collectionName: currentCollection.value?.name,
    });
  };

  const replaceDuplicates = (): void => {
    invoke('replace_duplicate_sprites', {
      sourceSprite: currentFrame.value,
    }).then(() => {
      const filteredSprites = changedSprites.value.filter(
        (sprite) =>
          sprite.id !== currentFrame.value?.id ||
          sprite.collectionName !== currentFrame.value?.collectionName
      );

      changedSprites.value = filteredSprites;
    });
  };

  const setCurrentSprite = (spriteName: string): void => {
    invoke('get_collection_from_sprite_name', {
      spriteName,
    }).then((collection: Collection) => {
      currentCollection.value = collection;
      inspectMode.value = InspectMode.Collection;
      // currentFrame.value = collection.sprites.find(
      //   (sprite) => sprite.name === spriteName
      // );
    });
  };

  const setLanguage = (language: string): void => {
    invoke('set_language', {
      language,
    });
  };

  const setMode = (mode: string): void => {
    invoke('set_mode', { mode });
  };

  return {
    version,
    allowedToPack,
    animationNames,
    changedSprites,
    currentAnimation,
    currentClip,
    currentCollection,
    currentCollections,
    currentFrame,
    mode,
    duplicateSprites,
    inspectMode,
    isPacking,
    packProgress,
    theme,

    cancelPack,
    changeMode,
    checkForChangedSprites,
    check,
    incrementFrameIndex,
    packCollections,
    replaceDuplicates,
    setCurrentSprite,
    setLanguage,
    setMode,
  };
});
