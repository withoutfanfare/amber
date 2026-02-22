<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { useProfileStore } from '@/stores/profiles'

const route = useRoute()
const profileStore = useProfileStore()

const navItems = [
  { name: 'Dashboard', path: '/', icon: 'home' },
  { name: 'Profiles', path: '/profiles', icon: 'database' },
  { name: 'Snapshots', path: '/snapshots', icon: 'camera' },
  { name: 'Storage', path: '/storage', icon: 'hard-drive' },
  { name: 'Settings', path: '/settings', icon: 'settings' },
  { name: 'Guide', path: '/guide', icon: 'book' },
]

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}

function handleProfileChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  if (value) profileStore.setActive(value)
}

onMounted(() => {
  profileStore.fetchAll()
})
</script>

<template>
  <aside class="sidebar titlebar-no-drag flex w-56 shrink-0 flex-col">
    <!-- Navigation -->
    <nav class="flex flex-1 flex-col gap-0.5 px-3 py-2">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors duration-150"
        :class="isActive(item.path) ? 'text-accent font-semibold' : 'text-text-secondary hover:text-text-primary hover:bg-surface-overlay'"
      >
        <!-- Home icon -->
        <svg v-if="item.icon === 'home'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
          <polyline points="9 22 9 12 15 12 15 22" />
        </svg>
        <!-- Database icon -->
        <svg v-else-if="item.icon === 'database'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M3 5V19A9 3 0 0 0 21 19V5" />
          <path d="M3 12A9 3 0 0 0 21 12" />
        </svg>
        <!-- Camera icon -->
        <svg v-else-if="item.icon === 'camera'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z" />
          <circle cx="12" cy="13" r="3" />
        </svg>
        <!-- Hard drive icon -->
        <svg v-else-if="item.icon === 'hard-drive'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="12" x2="2" y2="12" />
          <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
          <line x1="6" y1="16" x2="6.01" y2="16" />
          <line x1="10" y1="16" x2="10.01" y2="16" />
        </svg>
        <!-- Settings icon -->
        <svg v-else-if="item.icon === 'settings'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
        <!-- Book icon -->
        <svg v-else-if="item.icon === 'book'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
        </svg>
        <span>{{ item.name }}</span>
      </RouterLink>
    </nav>

    <!-- Profile selector -->
    <div v-if="profileStore.profiles.length > 0" class="border-t border-border-subtle px-3 py-3">
      <label class="text-[10px] font-semibold uppercase tracking-wide text-text-muted mb-1.5 block">
        Active Profile
      </label>
      <select
        :value="profileStore.activeProfileId ?? ''"
        class="control-field-sm w-full px-2 text-text-primary cursor-pointer appearance-none bg-no-repeat"
        style="background-image: url(&quot;data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23999999' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E&quot;); background-position: right 0.5rem center; background-repeat: no-repeat;"
        @change="handleProfileChange"
      >
        <optgroup
          v-for="[project, profiles] of profileStore.profilesByProject"
          :key="project"
          :label="project"
        >
          <option
            v-for="p in profiles"
            :key="p.id"
            :value="p.id"
          >
            {{ p.name }} ({{ p.dbType }}{{ p.environment ? ` · ${p.environment}` : '' }})
          </option>
        </optgroup>
      </select>
    </div>

    <!-- Status footer -->
    <div class="flex items-center gap-2 border-t border-border-subtle px-4 py-3">
      <span class="h-2 w-2 rounded-full bg-success"></span>
      <span class="text-xs text-text-tertiary">Service ready</span>
    </div>
  </aside>
</template>
