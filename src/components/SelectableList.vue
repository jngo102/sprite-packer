<template>
  <div class="selectable-list">
    <div class="row">
      <q-list class="list">
        <q-item>
          <q-item-section>
            <q-item-label>{{ props.title }}</q-item-label>
          </q-item-section>
        </q-item>
        <q-item v-for="item in props.items" :key="item" clickable v-ripple @click="selectItem(item)">
          <q-item-section>
            <q-item-label>{{ item }}</q-item-label>
          </q-item-section>
        </q-item>
      </q-list>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

export interface SelectableListProps {
  items: Array<string>;
  onSelectItem: (item: string) => void;
  selectedItem: string;
  title: string;
}

const emit = defineEmits(['selectItem']);

const props = defineProps<SelectableListProps>();

const getHeight = computed((): number => {
  var height = window.innerHeight;
  const clipPreview = document.getElementById('clip-preview');
  if (clipPreview) {
    height -= clipPreview.clientHeight;
  }
  const navbar = document.getElementById('navbar');
  if (navbar) {
    height -= navbar.clientHeight;
  }
  height -= 64;
  height = Math.max(height, 256);
  return height;
});

const selectItem = (item: string) => {
  emit('selectItem', item);
};
</script>

<style scoped lang="sass">
.list
  height: v-bind('getHeight')
  overflow-y: auto
</style>
