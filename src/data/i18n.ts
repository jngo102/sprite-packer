import LocalizedStrings from 'react-localization'

const languages = {
    en: {
        animations: "Animations",
        atlases: "Atlases",
        cancel: "Cancel",
        clips: "Clips",
        frames: "Frames",
        language: "Language",
        pack: "Pack",
        packing: "Packing ",
        quit: "Quit",
        refresh: "Refresh",
        setSpritesPath: "Set Sprites Path"
    },
    cn: {
        animations: "动画",
        atlases: "图集",
        cancel: "取消",
        clips: "剪辑",
        frames: "帧",
        language: "语言",
        pack: "打包",
        packing: "打包中",
        quit: "退出",
        refresh: "刷新",
        setSpritesPath: "设置精灵路径"
    },
    es: {
        animations: "Animaciones",
        atlases: "Atlas",
        cancel: "Cancelar",
        clips: "Clips",
        frames: "Fotogramas",
        language: "Idioma",
        pack: "Paquete",
        packing: "Empaquetando",
        quit: "Salir",
        refresh: "Refrescar",
        setSpritesPath: "Establecer ruta de sprites"
    },
    fr: {
        animations: "Animations",
        atlases: "Atlas",
        cancel: "Annuler",
        clips: "Vitesses",
        frames: "Images",
        language: "Langue",
        pack: "Pack",
        packing: "Packaging",
        quit: "Quitter",
        refresh: "Rafraîchir",
        setSpritesPath: "Définir le chemin des sprites"
    },
    de: {
        animations: "Animationen",
        atlases: "Atlanten",
        cancel: "Abbrechen",
        clips: "Clips",
        frames: "Frames",
        language: "Sprache",
        pack: "Packen",
        packing: "Packen ",
        quit: "Beenden",
        refresh: "Aktualisieren",
        setSpritesPath: "Pfad für Sprites festlegen"
    }
}
const i18n = new LocalizedStrings(languages)

export { i18n, languages }