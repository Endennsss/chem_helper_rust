<template>
  <div class="app-shell">
    <Transition name="splash-fade">
      <div v-if="showSplash" class="splash-screen">
        <div class="splash-content">
          <img src="/icon.svg" alt="SS14 Helper" class="splash-logo" />
          <h1 class="splash-title">Chem Helper Rust</h1>
          <p class="splash-subtitle">Chemistry Console</p>
          <div class="splash-loader">
            <div class="splash-loader-bar"></div>
          </div>
          <div class="splash-status">{{ splashStatus }}</div>
        </div>
      </div>
    </Transition>

    <header class="app-chrome">
      <div class="chrome-left">
        <div class="brand-block">
          <img src="/icon.svg" alt="" class="brand-logo" width="43" height="43" />
          <div>
            <p>SS14 Chemistry Console</p>
            <h1>Chem Helper Rust</h1>
          </div>
        </div>
      </div>

      <div class="chrome-right">
        <nav class="app-tabs" aria-label="Разделы приложения">
          <button type="button" :class="{ active: activeTab === 'chemistry' }" @click="activeTab = 'chemistry'">
            <FlaskConical :size="16" />
            Химия
          </button>
          <button type="button" :class="{ active: activeTab === 'settings' }" @click="activeTab = 'settings'">
            <Settings :size="16" />
            Настройки
          </button>
          <button type="button" :class="{ active: activeTab === 'guide' }" @click="activeTab = 'guide'">
            <BadgeInfo :size="16" />
            Гайд
          </button>
        </nav>
        <div class="external-actions">
          <a href="https://fish.station.wiki.shizainc.com/ru/home" target="_blank" rel="noreferrer" title="Открыть wiki">
            <BookOpen :size="16" />
            Wiki
          </a>
          <a href="https://discord.gg/EfHKrSEP8X" target="_blank" rel="noreferrer" title="Открыть Discord">
            <MessageCircle :size="16" />
            Discord
          </a>
        </div>
        <button class="icon-button" type="button" title="Обновить расчет" @click="refreshCalculation">
          <RefreshCw :size="17" />
        </button>
      </div>
    </header>

    <main v-if="activeTab === 'chemistry'" class="workspace">
      <aside class="sidebar">
        <section class="panel composer-panel">
          <header class="panel-heading">
            <FlaskConical :size="18" />
            <span>Добавить реакцию</span>
          </header>

          <div class="field-stack">
            <label for="quantity">{{ quantityLabel }}</label>
            <div class="quantity-control">
              <button type="button" class="step-button" title="Уменьшить" @click="nudgeQuantity(-5)">
                <Minus :size="16" />
              </button>
              <input
                id="quantity"
                v-model.number="quantity"
                type="number"
                :min="selectedRecipe?.quantized && selectedRecipe.unit === 'реакц.' ? 1 : 0.000001"
                :step="selectedRecipe?.quantized && selectedRecipe.unit === 'реакц.' ? 1 : 'any'"
                max="1000000000"
                inputmode="decimal"
                @keydown.enter.prevent="focusMaterial"
              />
              <button type="button" class="step-button" title="Увеличить" @click="nudgeQuantity(5)">
                <Plus :size="16" />
              </button>
            </div>
          </div>

          <div class="field-stack combo-field">
            <label for="material">Рецепт</label>
            <div class="combo-box">
              <Search :size="17" />
              <input
                id="material"
                ref="materialInput"
                v-model="materialQuery"
                type="text"
                autocomplete="off"
                placeholder="Рецепт или исходный реагент"
                @focus="suggestionsOpen = true"
                @blur="closeSuggestions"
                @keydown.enter.prevent="addMaterial"
                @keydown.down.prevent="moveSuggestion(1)"
                @keydown.up.prevent="moveSuggestion(-1)"
              />
            </div>

            <Transition name="dropdown">
              <div v-if="suggestionsOpen && filteredMaterials.length" class="suggestions">
                <button
                  v-for="(recipe, index) in filteredMaterials"
                  :key="recipe.nameEn"
                  type="button"
                  :class="{ active: index === activeSuggestion }"
                  @mousedown.prevent="selectMaterial(recipe)"
                >
                  <span>{{ recipe.name }}</span>
                  <small>{{ recipe.ingredientCount }} реаг.</small>
                </button>
              </div>
            </Transition>
          </div>

          <p v-if="selectedRecipe?.quantized && selectedRecipe.unit === 'u'" class="recipe-hint">Целая порция: {{ formatQuantity(selectedRecipe.batchSize) }}u. Объём округляется вверх до целой порции.</p>
          <p v-if="selectedRecipe?.quantized && selectedRecipe.unit === 'реакц.'" class="recipe-hint">Введите целое число порций реакции.</p>
          <button class="primary-action" type="button" @click="addMaterial">
            <Plus :size="18" />
            Добавить
          </button>
        </section>

        <section class="panel library-panel">
          <header class="panel-heading">
            <Library :size="18" />
            <span>Каталог</span>
            <b>{{ materials.length }}</b>
          </header>
          <label class="library-filter" for="recipe-category">
            <span>Категория</span>
            <select id="recipe-category" v-model="selectedCategory">
              <option value="">Все категории ({{ materials.length }})</option>
              <option v-for="category in categoryOptions" :key="category.name" :value="category.name">{{ category.name }} ({{ category.count }})</option>
            </select>
          </label>
          <p class="catalog-state">{{ sourceLabel }} · {{ libraryMaterials.length }} из {{ materials.length }}</p>
          <div class="library-list">
            <button
              v-for="recipe in libraryMaterials"
              :key="recipe.nameEn"
              type="button"
              :data-reaction-id="recipe.reactionId"
              @click="selectMaterial(recipe)"
              @mouseenter="showTooltip(recipe, $event)"
              @mouseleave="hideTooltip"
            >
              <span>{{ recipe.name }}</span>
              <Thermometer v-if="recipe.temperature != null || recipe.maxTemperature != null" :size="13" />
            </button>
          </div>
        </section>

        <section class="panel favorites-panel">
          <header class="panel-heading">
            <Star :size="18" />
            <span>Избранное</span>
            <b>{{ favorites.length }}</b>
          </header>

          <TransitionGroup v-if="favorites.length" name="row" tag="div" class="favorite-list">
            <div v-for="(item, index) in favorites" :key="favoriteKey(item, index)" class="favorite-row">
              <button type="button" class="row-main" @click="useFavorite(item)">
                <Play :size="15" />
                <span>{{ formatQuantity(item.quantity) }}{{ item.unit || 'u' }} {{ item.name }}</span>
              </button>
              <button
                class="icon-button compact"
                type="button"
                title="Удалить из избранного"
                @click="removeFavorite(index)"
              >
                <X :size="15" />
              </button>
            </div>
          </TransitionGroup>

          <div v-else class="empty-state">Нет сохраненных рецептов</div>
        </section>
      </aside>

      <section class="workbench">
        <div class="command-strip">
          <div class="metric">
            <span>Задач</span>
            <strong>{{ production.length }}</strong>
          </div>
          <div class="metric">
            <span>Реагентов</span>
            <strong>{{ activeResultCount }}</strong>
          </div>
          <div class="metric">
            <span>Всего</span>
            <strong>{{ totalRequested }}</strong>
          </div>

          <button class="ghost-action" type="button" :disabled="!production.length" @click="clearQueue">
            <Trash2 :size="17" />
            Очистить
          </button>
        </div>

        <div class="grid-panels">
          <section class="panel queue-panel">
            <header class="panel-heading">
              <Factory :size="18" />
              <span>Очередь</span>
            </header>

            <TransitionGroup v-if="queuedItems.length" name="row" tag="div" class="queue-list">
              <article v-for="item in queuedItems" :key="item.key" class="queue-row">
                <div class="amount-badge">{{ formatQuantity(item.quantity) }}{{ item.unit || 'u' }}</div>
                <div class="queue-name">{{ item.name }}</div>
                <div class="row-actions">
                  <button
                    class="icon-button compact favorite-toggle"
                    :class="{ active: isFavorite(item) }"
                    type="button"
                    :title="isFavorite(item) ? 'В избранном' : 'В избранное'"
                    @click="addFavorite(item)"
                  >
                    <Star :size="15" :fill="isFavorite(item) ? 'currentColor' : 'none'" />
                  </button>
                  <button class="icon-button compact" type="button" title="Удалить" @click="removeItem(item.index)">
                    <X :size="15" />
                  </button>
                </div>
              </article>
            </TransitionGroup>

            <div v-else class="empty-state">Очередь пуста</div>
          </section>

          <section class="panel results-panel">
            <header class="panel-heading split-heading">
              <div>
                <Beaker :size="18" />
                <span>Итоги</span>
              </div>
              <div class="segmented">
                <button
                  type="button"
                  :class="{ active: activeResult === 'direct' }"
                  @click="activeResult = 'direct'"
                >
                  Компоненты
                </button>
                <button
                  type="button"
                  :class="{ active: activeResult === 'base' }"
                  @click="activeResult = 'base'"
                >
                  Исходные
                </button>
              </div>
            </header>

            <Transition mode="out-in" name="switch">
              <div :key="activeResult" class="result-list">
                <div v-if="busy" class="loading-line">Расчет</div>

                <template v-else-if="activeRows.length">
                  <div v-for="row in activeRows" :key="`${activeResult}-${row.name}`" class="result-row">
                    <span>{{ row.name }}</span>
                    <strong>{{ formatQuantity(row.quantity) }}u</strong>
                  </div>
                </template>

                <div v-else class="empty-state">Нет данных для расчета</div>
              </div>
            </Transition>
          </section>

          <section class="panel guide-panel">
            <header class="panel-heading">
              <ListChecks :size="18" />
              <span>Пошаговый состав</span>
            </header>

            <div v-if="calculation.guides.length" class="guide-list">
              <article
                v-for="(guide, guideIndex) in calculation.guides"
                :key="guideKey(guide, guideIndex)"
                class="guide-block"
              >
                <button class="guide-title" type="button" @click="toggleGuide(guide, guideIndex)">
                  <span>{{ formatQuantity(guide.quantity) }}{{ guide.unit || 'u' }} {{ guide.name }}</span>
                  <ChevronDown :class="{ open: isGuideOpen(guide, guideIndex) }" :size="18" />
                </button>

                <Transition name="collapse">
                  <div v-if="isGuideOpen(guide, guideIndex)" class="step-list">
                    <div v-for="(step, stepIndex) in guide.steps" :key="`${step.product}-${stepIndex}`" class="step-row">
                      <div class="step-index">{{ stepIndex + 1 }}</div>
                      <div class="step-body">
                        <div class="step-title">
                          <span>{{ step.product }}</span>
                          <strong>{{ formatQuantity(step.quantity) }}{{ step.unit || 'u' }}</strong>
                          <em v-if="step.temperature != null || step.maxTemperature != null">
                            <Thermometer :size="13" />
                            {{ temperatureLabel(step.temperature, step.maxTemperature) }}
                          </em>
                        </div>
                        <div class="ingredient-grid">
                          <span v-for="ingredient in step.ingredients" :key="ingredient.name">
                            {{ ingredient.name }}
                            <b>{{ formatQuantity(ingredient.quantity) }}u</b>
                          </span>
                        </div>
                        <div v-if="step.products?.length" class="reaction-outputs">
                          <span class="reaction-detail-label">Выход</span>
                          <div class="ingredient-grid">
                            <span v-for="product in step.products" :key="product.name">{{ product.name }} <b>{{ formatQuantity(product.quantity) }}u</b></span>
                          </div>
                        </div>
                        <ul v-if="step.conditions?.length || step.effects?.length" class="reaction-conditions">
                          <li v-for="condition in [...(step.conditions || []), ...(step.effects || [])]" :key="condition">{{ condition }}</li>
                        </ul>
                        <div v-if="step.catalysts?.length" class="ingredient-grid">
                          <span v-for="catalyst in step.catalysts" :key="catalyst.name">
                            Катализатор: {{ catalyst.name }}
                            <b>{{ catalyst.quantity }}u · не расходуется</b>
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                </Transition>
              </article>
            </div>

            <div v-else class="empty-state">Добавьте препарат для состава</div>
          </section>
        </div>
      </section>
    </main>

    <main v-else-if="activeTab === 'settings'" class="settings-workspace">
      <section class="panel settings-panel">
        <header class="panel-heading">
          <Settings :size="18" />
          <span>Настройки</span>
        </header>

        <div class="settings-section">
          <h3>Данные рецептов</h3>
          <p class="settings-hint">{{ sourceLabel }} · {{ materials.length }} рецептов</p>
          <p v-if="catalogWarning" class="settings-hint">{{ catalogWarning }}</p>
          <p v-else class="settings-hint">Сохранённые рецепты доступны без интернета. Обновление выполняется в фоне.</p>
          <button class="primary-action" type="button" :disabled="catalogUpdating" @click="updateOnlineCatalog(false)">
            <RefreshCw :size="16" />
            {{ catalogUpdating ? 'Обновление…' : 'Обновить каталог' }}
          </button>
        </div>

        <div class="settings-section">
          <div class="setting-row">
            <div>
              <h3>Анимации интерфейса</h3>
              <p class="settings-hint">Снижает нагрузку на GPU и ускоряет работу интерфейса</p>
            </div>
            <label class="toggle-switch">
              <input v-model="animationsEnabled" type="checkbox" />
              <span class="toggle-track" aria-hidden="true"></span>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <div class="setting-row">
            <div>
              <h3>Экран загрузки</h3>
              <p class="settings-hint">Показывать заставку при загрузке приложения</p>
            </div>
            <label class="toggle-switch">
              <input v-model="splashEnabled" type="checkbox" />
              <span class="toggle-track" aria-hidden="true"></span>
            </label>
          </div>
        </div>

        <!-- <div class="settings-section">
          <div class="setting-row">
            <div>
              <h3>Анимации интерфейса</h3>
              <p class="settings-hint">Снижает нагрузку на GPU и ускоряет работу интерфейса</p>
            </div>
            <label class="toggle-switch">
              <input v-model="animationsEnabled" type="checkbox" />
              <span class="toggle-track" aria-hidden="true"></span>
            </label>
          </div>
        </div> Должна быть смена языка скоро -->

      </section>
    </main>

    <main v-else class="guide-workspace">
      <section class="panel guide-page">
        <header class="panel-heading">
          <BadgeInfo :size="18" />
          <span>Гайды</span>
        </header>

        <div class="guide-placeholder">

          <h2>Раздел находится в разработке</h2>

          <p>
            Здесь будет возможно полезная информация.
          </p>

          <div class="guide-note">
            Пока что раздел пуст.
          </div>

          <div>
            <img src="/smile.jpg" alt="" width="200" height="200">
          </div>

        </div>
      </section>
    </main>

    <Transition name="toast">
      <div v-if="toast" class="toast" :class="toast.type">
        <AlertTriangle :size="17" />
        {{ toast.text }}
      </div>
    </Transition>

    <Transition name="fade">
      <div
        v-if="tooltipItem"
        class="tooltip-popup"
        :style="{ left: tooltipPosition.x + 'px', top: tooltipPosition.y + 'px' }"
      >
        <div class="tooltip-header">
          <div class="tooltip-title">{{ tooltipItem.name }}</div>
          <div v-if="formatColor(tooltipItem.color)" class="tooltip-color" :style="{ backgroundColor: formatColor(tooltipItem.color) }"></div>
        </div>
        <div v-if="tooltipItem.description" class="tooltip-description">{{ tooltipItem.description }}</div>
        <div class="tooltip-description">{{ tooltipItem.category }} · {{ tooltipItem.reactionId }}</div>
        <div v-if="tooltipItem.ingredientNames?.length" class="tooltip-description">Компоненты: {{ tooltipItem.ingredientNames.join(', ') }}</div>
        <div v-if="tooltipItem.products?.length" class="tooltip-description">Выход порции: {{ tooltipItem.products.map(p => `${formatQuantity(p.quantity)}u ${p.name}`).join(', ') }}</div>
        <ul v-if="tooltipItem.conditions?.length || tooltipItem.reactionEffects?.length" class="reaction-conditions">
          <li v-for="line in [...(tooltipItem.conditions || []), ...(tooltipItem.reactionEffects || [])]" :key="line">{{ line }}</li>
        </ul>
        <div v-if="tooltipItem.effects && tooltipItem.effects.length" class="tooltip-effects">
          <strong>Эффекты:</strong>
          <ul>
            <li v-for="(effect, index) in tooltipItem.effects" :key="index">{{ effect }}</li>
          </ul>
        </div>
        <div v-if="tooltipItem.temperature != null || tooltipItem.maxTemperature != null" class="tooltip-temp">
          <Thermometer :size="13" />
          Температура: {{ temperatureLabel(tooltipItem.temperature, tooltipItem.maxTemperature) }}
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import {
  AlertTriangle,
  Beaker,
  BookOpen,
  ChevronDown,
  Factory,
  FlaskConical,
  Library,
  ListChecks,
  Settings,
  MessageCircle,
  Minus,
  Play,
  Plus,
  RefreshCw,
  Search,
  Star,
  Thermometer,
  Trash2,
  X,
  BadgeInfo,
} from '@lucide/vue';
import { calculateProduction, loadMaterials, refreshMaterialsFromSource } from '../services/api';
import { storage } from '../services/storage';
import { categories, filterRecipes, sumUnits, migrateSavedItems, temperatureLabel } from '../services/catalog-view';

const EMPTY_CALCULATION = {
  items: [],
  direct: [],
  base: [],
  guides: [],
};

const materials = ref([]);
const materialAliases = ref({});
const selectedCategory = ref(storage.getItem('recipeCategory') || '');
const categoryOptions = computed(() => categories(materials.value));
const selectedRecipe = computed(() => materials.value.find(r => r.name === materialQuery.value) || (materialQuery.value.trim() ? filteredMaterials.value[activeSuggestion.value] : null));
const quantityLabel = computed(() => selectedRecipe.value?.unit === 'реакц.' ? 'Число порций реакции' : selectedRecipe.value?.products?.length > 1 ? 'Объём смеси, u' : 'Объём продукта, u');
const production = ref(readStorage('productionItems', []));
const favorites = ref(readStorage('favoriteItems', []));
const calculation = ref(EMPTY_CALCULATION);
const expandedGuides = ref(new Set());

const quantity = ref(Number(storage.getItem('lastQuantity')) || 30);
const materialQuery = ref('');
const materialInput = ref(null);
const suggestionsOpen = ref(false);
const activeSuggestion = ref(0);
const activeTab = ref('chemistry');
const animationsEnabled = ref(storage.getItem('animationsEnabled') !== 'false');
const splashEnabled = ref(storage.getItem('splashEnabled') !== 'false');
const activeResult = ref('direct');
const apiState = ref('loading');
const catalogSource = ref('builtin');
const catalogUpdating = ref(false);
const catalogWarning = ref('');
const sourceLabel = computed(() => ({ builtin: 'Рецепты Sunrise', bundled: 'Каталог Sunrise', cache: 'Сохранённый Sunrise', online: 'Актуальный Sunrise' }[catalogSource.value] || catalogSource.value));
const busy = ref(false);
const toast = ref(null);
const tooltipItem = ref(null);
const tooltipPosition = ref({ x: 0, y: 0 });
const showSplash = ref(storage.getItem('splashEnabled') !== 'false');
const splashStatus = ref('Инициализация...');

let toastTimer = 0;
let calculationRun = 0;
let tooltipTimer = 0;

const normalizedQuery = computed(() => materialQuery.value.trim().toLocaleLowerCase('ru'));

const libraryMaterials = computed(() => filterRecipes(materials.value, selectedCategory.value, normalizedQuery.value));
const filteredMaterials = computed(() => libraryMaterials.value.slice(0, 10));

const queuedItems = computed(() =>
  production.value
    .map((item, index) => ({
      ...item,
      index,
      key: `${item.name}-${item.quantity}-${index}`,
    }))
    .reverse(),
);

const activeRows = computed(() => calculation.value[activeResult.value] || []);
const activeResultCount = computed(() => activeRows.value.length);
const totalRequested = computed(() => {
  const { volume, batches } = sumUnits(production.value);
  return [volume ? `${formatQuantity(volume)}u` : '', batches ? `${formatQuantity(batches)} порц.` : ''].filter(Boolean).join(' + ') || '0u';
});
watch(selectedCategory, value => { activeSuggestion.value = 0; storage.setItem('recipeCategory', value); });

watch(
  production,
  () => {
    storage.setItem('productionItems', JSON.stringify(production.value));
    refreshCalculation();
  },
  { deep: true, flush: 'sync' },
);

watch(
  favorites,
  () => {
    storage.setItem('favoriteItems', JSON.stringify(favorites.value));
  },
  { deep: true, flush: 'sync' },
);

watch(normalizedQuery, () => {
  activeSuggestion.value = 0;
});

watch(animationsEnabled, (value) => {
  storage.setItem('animationsEnabled', String(value));
  document.documentElement.classList.toggle('no-animations', !value);
});

watch(splashEnabled, (value) => {
  storage.setItem('splashEnabled', String(value));
});

watch(
  () => calculation.value.guides,
  (guides) => {
    expandedGuides.value = guides.length ? new Set([guideKey(guides[0], 0)]) : new Set();
  },
);

onMounted(async () => {
  window.addEventListener('storage-error', event => showToast(event.detail, 'error'));
  document.documentElement.classList.toggle('no-animations', !animationsEnabled.value);

  if (splashEnabled.value) {
    splashStatus.value = 'Загрузка материалов...';
    await refreshMaterials();

    splashStatus.value = 'Подготовка данных...';
    await refreshCalculation();

    splashStatus.value = 'Готово!';
    await new Promise(resolve => setTimeout(resolve, 300));

    showSplash.value = false;
  } else {
    showSplash.value = false;
    await refreshMaterials();
    await refreshCalculation();
  }
  updateOnlineCatalog(true);
});

function readStorage(key, fallback) {
  try {
    const value = JSON.parse(storage.getItem(key) || 'null');
    return Array.isArray(value) ? value : fallback;
  } catch {
    return fallback;
  }
}

function applyCatalog(payload) {
  materials.value = payload.materials;
  materialAliases.value = payload.aliases || {};
  catalogSource.value = payload.source;
  if (selectedCategory.value && !categoryOptions.value.some(c => c.name === selectedCategory.value)) selectedCategory.value = '';
  const queue = migrateSavedItems(production.value, materials.value, materialAliases.value);
  const saved = migrateSavedItems(favorites.value, materials.value, materialAliases.value);
  if (JSON.stringify(queue) !== JSON.stringify(production.value)) production.value = queue;
  if (JSON.stringify(saved) !== JSON.stringify(favorites.value)) favorites.value = saved;
}

async function refreshMaterials() {
  apiState.value = 'loading';
  try {
    const payload = await loadMaterials();
    applyCatalog(payload);
    apiState.value = 'ready';
  } catch (error) {
    apiState.value = 'error';
    showToast(error.message || String(error) || 'Каталог недоступен', 'error');
  }
}

async function updateOnlineCatalog(quiet = false) {
  if (catalogUpdating.value) return;
  catalogUpdating.value = true;
  catalogWarning.value = '';
  try {
    const payload = await refreshMaterialsFromSource();
    applyCatalog(payload);
    await refreshCalculation();
    if (!quiet) showToast(`Каталог обновлён: ${payload.count} рецептов`, 'info');
  } catch (error) {
    catalogWarning.value = `Обновление недоступно. Используется сохранённый каталог. ${error}`;
    if (!quiet) showToast(catalogWarning.value, 'error');
  } finally { catalogUpdating.value = false; }
}

async function refreshCalculation() {
  const run = ++calculationRun;

  if (!production.value.length) {
    calculation.value = EMPTY_CALCULATION;
    busy.value = false;
    return;
  }

  busy.value = true;
  try {
    const payload = await calculateProduction(production.value);
    if (run !== calculationRun) return;
    calculation.value = payload;
    apiState.value = 'ready';
  } catch (error) {
    if (run !== calculationRun) return;
    apiState.value = 'error';
    calculation.value = EMPTY_CALCULATION;
    showToast(error.message || String(error) || 'Ошибка расчета', 'error');
  } finally {
    if (run === calculationRun) {
      busy.value = false;
    }
  }
}

function findRecipe(name) {
  const needle = name.trim().toLocaleLowerCase('ru');
  return materials.value.find((recipe) => recipe.name.toLocaleLowerCase('ru') === needle);
}

function addMaterial() {
  const amount = Number(quantity.value);
  const query = materialQuery.value.trim();
  const selected = filteredMaterials.value[activeSuggestion.value];
  const recipe = query ? findRecipe(query) || selected : null;

  if (!Number.isFinite(amount) || amount <= 0 || amount > 1_000_000_000) {
    showToast('Введите количество больше 0 и не больше 1 000 000 000', 'error');
    return;
  }

  if (!recipe) {
    showToast('Препарат не найден', 'error');
    return;
  }

  if (recipe.quantized && recipe.unit === 'реакц.' && Math.abs(amount % 1) > 1e-9) {
    showToast('Для этой реакции нужно целое число порций', 'error');
    return;
  }

  production.value.push({
    name: recipe.name,
    id: recipe.nameEn,
    unit: recipe.unit,
    quantity: amount,
  });

  storage.setItem('lastQuantity', String(amount));
  materialQuery.value = '';
  activeSuggestion.value = 0;
  suggestionsOpen.value = false;
  nextTick(() => materialInput.value?.focus());
}

function removeItem(index) {
  production.value.splice(index, 1);
}

function clearQueue() {
  production.value = [];
}

function addFavorite(item) {
  if (isFavorite(item)) {
    showToast('Рецепт уже в избранном', 'info');
    return;
  }

  favorites.value.push({
    name: item.name,
    id: item.id,
    unit: item.unit || 'u',
    quantity: item.quantity,
  });
}

function isFavorite(item) {
  return favorites.value.some(
    (favorite) => (favorite.id || favorite.name) === (item.id || item.name) && Number(favorite.quantity) === Number(item.quantity),
  );
}

function removeFavorite(index) {
  favorites.value.splice(index, 1);
}

function useFavorite(item) {
  if (!materials.value.some(r => r.nameEn === item.id)) {
    showToast('Этот старый рецепт отсутствует в текущем билде Sunrise', 'error');
    return;
  }
  production.value.push({
    name: item.name,
    id: item.id,
    unit: item.unit || 'u',
    quantity: Number(item.quantity),
  });
}

function selectMaterial(recipe) {
  const name = typeof recipe === 'string' ? recipe : recipe.name;
  materialQuery.value = name;
  activeSuggestion.value = 0;
  suggestionsOpen.value = false;
  nextTick(() => materialInput.value?.focus());
}

function closeSuggestions() {
  window.setTimeout(() => {
    suggestionsOpen.value = false;
  }, 120);
}

function moveSuggestion(direction) {
  if (!filteredMaterials.value.length) return;
  suggestionsOpen.value = true;
  const next = activeSuggestion.value + direction;
  activeSuggestion.value = (next + filteredMaterials.value.length) % filteredMaterials.value.length;
}

function nudgeQuantity(delta) {
  const next = Number(quantity.value || 0) + delta;
  quantity.value = Math.max(1, next);
}

function focusMaterial() {
  materialInput.value?.focus();
}

function favoriteKey(item, index) {
  return `${item.name}-${item.quantity}-${index}`;
}

function guideKey(guide, index) {
  return `${guide.name}-${guide.quantity}-${index}`;
}

function isGuideOpen(guide, index) {
  return expandedGuides.value.has(guideKey(guide, index));
}

function toggleGuide(guide, index) {
  const key = guideKey(guide, index);
  const next = new Set(expandedGuides.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  expandedGuides.value = next;
}

function formatQuantity(value) {
  const number = Number(value || 0);
  if (Number.isInteger(number)) return String(number);
  return number.toFixed(6).replace(/\.?0+$/, '');
}

function formatTemperature(value) {
  const number = Number(value || 0);
  return Number.isInteger(number) ? String(number) : number.toFixed(6);
}

function showToast(text, type = 'info') {
  window.clearTimeout(toastTimer);
  toast.value = { text, type };
  toastTimer = window.setTimeout(() => {
    toast.value = null;
  }, 2600);
}

function showTooltip(item, event) {
  window.clearTimeout(tooltipTimer);
  tooltipTimer = window.setTimeout(() => {
    if (item.description || item.color || item.effects || item.ingredientNames) {
      tooltipItem.value = item;

      nextTick(() => {
        const padding = 20;
        const tooltipWidth = 460;
        const maxTooltipHeight = window.innerHeight - 100;

        let x = event.clientX;
        let y = event.clientY;

        if (x + tooltipWidth / 2 + padding > window.innerWidth) {
          x = window.innerWidth - tooltipWidth / 2 - padding;
        }
        if (x - tooltipWidth / 2 < padding) {
          x = tooltipWidth / 2 + padding;
        }

        const spaceAbove = y - padding;
        const spaceBelow = window.innerHeight - y - padding;

        if (spaceAbove > maxTooltipHeight / 2 || spaceAbove > spaceBelow) {
          y = Math.max(y, maxTooltipHeight + padding + 16);
        } else {
          y = Math.min(y + 40, window.innerHeight - padding);
        }

        tooltipPosition.value = { x, y };
      });
    }
  }, 500);
}

function hideTooltip() {
  window.clearTimeout(tooltipTimer);
  tooltipItem.value = null;
}

function formatColor(color) {
  if (!color) return null;
  if (typeof color === 'string' && color.startsWith('#')) return color;
  return null;
}
</script>
