import LocalizedStrings from 'react-localization'

const languages = {
    en: {
        animations: "Animations",
        atlases: "Atlases",
        cancel: "Cancel",
        changedSprites: "Changed Sprites",
        clips: "Clips",
        duplicates: "Duplicates",
        findDuplicates: "Find Duplicates",
        frames: "Frames",
        language: "Language",
        pack: "Pack",
        packing: "Packing ",
        quit: "Quit",
        refresh: "Refresh",
        replaceDuplicates: "Replace Duplicates",
        setSpritesPath: "Set Sprites Path"
    },
    cn: {
        animations: "动画",
        atlases: "图集",
        cancel: "取消",
        changedSprites: "更改的精灵",
        clips: "剪辑",
        duplicates: "重复项",
        findDuplicates: "查找重复项",
        frames: "帧",
        language: "语言",
        pack: "打包",
        packing: "打包中",
        quit: "退出",
        refresh: "刷新",
        replaceDuplicates: "替换重复项",
        setSpritesPath: "设置精灵路径"
    },
    es: {
        animations: "Animaciones",
        atlases: "Atlas",
        cancel: "Cancelar",
        changedSprites: "Sprites cambiados",
        clips: "Clips",
        duplicates: "Duplicates",
        findDuplicates: "Encontrar Duplicates",
        frames: "Fotogramas",
        language: "Idioma",
        pack: "Paquete",
        packing: "Empaquetando",
        quit: "Salir",
        refresh: "Refrescar",
        replaceDuplicates: "Reemplazar Duplicates",
        setSpritesPath: "Establecer ruta de sprites"
    },
    fr: {
        animations: "Animations",
        atlases: "Atlas",
        cancel: "Annuler",
        changedSprites: "Sprites modifiés",
        clips: "Vitesses",
        duplicates: "Duplicates",
        findDuplicates: "Trouver Duplicates",
        frames: "Images",
        language: "Langue",
        pack: "Pack",
        packing: "Packaging",
        quit: "Quitter",
        refresh: "Rafraîchir",
        replaceDuplicates: "Remplacer Duplicates",
        setSpritesPath: "Définir le chemin des sprites"
    },
    de: {
        animations: "Animationen",
        atlases: "Atlanten",
        cancel: "Abbrechen",
        changedSprites: "Geänderte Sprites",
        clips: "Clips",
        duplicates: "Duplicates",
        findDuplicates: "Duplicates finden",
        frames: "Frames",
        language: "Sprache",
        pack: "Packen",
        packing: "Packen ",
        quit: "Beenden",
        refresh: "Aktualisieren",
        replaceDuplicates: "Duplicates ersetzen",
        setSpritesPath: "Pfad für Sprites festlegen"
    }
}
const i18n = new LocalizedStrings(languages)

export { i18n, languages }