import { invoke } from '@tauri-apps/api/core';
export const loadMaterials = () => invoke('load_materials');
export const calculateProduction = items => invoke('calculate_production', { items: items.map(item => ({ name: item.id || item.name, quantity: item.quantity })) });
export const refreshMaterialsFromSource = () => invoke('refresh_materials');
