import test from 'node:test';
import assert from 'node:assert/strict';
import { categories,filterRecipes,sumUnits,migrateSavedItems } from '../src/services/catalog-view.js';
const recipes=[
  {name:'Коктейль',nameEn:'reaction:Drink',reactionId:'Drink',category:'Напитки',unit:'u',ingredientNames:['Сок']},
  {name:'Бикаридин · вариант 1',nameEn:'reaction:MedicineOne',category:'Медицина',unit:'u'},
  {name:'Бикаридин · вариант 2',nameEn:'reaction:MedicineTwo',category:'Медицина',unit:'u'},
  {name:'Пена',nameEn:'reaction:Foam',category:'Особые реакции',unit:'реакц.'},
];
test('counts every variant in its category and filters by category plus ingredient',()=>{
  assert.deepEqual(categories(recipes),[{name:'Медицина',count:2},{name:'Напитки',count:1},{name:'Особые реакции',count:1}]);
  assert.equal(filterRecipes(recipes,'Медицина','бикаридин').length,2);
  assert.equal(filterRecipes(recipes,'Напитки','сок')[0].nameEn,'reaction:Drink');
  assert.equal(filterRecipes(recipes,'','Foam')[0].name,'Пена');
});
test('batch counts are not added to liquid volume',()=>{
  assert.deepEqual(sumUnits([{quantity:30},{quantity:0.5,unit:'u'},{quantity:2,unit:'реакц.'}]),{volume:30.5,batches:2});
});
test('upgrade preserves favorites and maps old product IDs to a real reaction',()=>{
  const items=[{id:'Bicaridine',name:'Бикаридин',quantity:30},{id:'Removed',name:'Старый рецепт',quantity:10}];
  const migrated=migrateSavedItems(items,recipes,{Bicaridine:'reaction:MedicineOne'});
  assert.deepEqual(migrated[0],{id:'reaction:MedicineOne',name:'Бикаридин · вариант 1',quantity:30,unit:'u'});
  assert.deepEqual(migrated[1],items[1]);
  assert.equal(items[0].id,'Bicaridine');
});
test('shows cold-only synthesis requirements and bounded temperature ranges',async()=>{
  const { temperatureLabel } = await import('../src/services/catalog-view.js');
  assert.equal(temperatureLabel(null,300),'≤ 300K');
  assert.equal(temperatureLabel(370,null),'≥ 370K');
  assert.equal(temperatureLabel(300,400),'300–400K');
  assert.equal(temperatureLabel(null,null),'');
  assert.equal(temperatureLabel(0,300),'0–300K');
});
