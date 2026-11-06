export function categories(recipes) {
  const counts=new Map();
  for (const recipe of recipes) {
    const name=recipe.category || 'Особые реакции';
    counts.set(name,(counts.get(name)||0)+1);
  }
  return [...counts].map(([name,count])=>({ name,count })).sort((a,b)=>a.name.localeCompare(b.name,'ru'));
}
export function filterRecipes(recipes,category,query) {
  const key=query.trim().toLocaleLowerCase('ru');
  return recipes.filter(recipe => (!category || recipe.category===category) && (!key || [recipe.name,recipe.nameEn,recipe.reactionId,...(recipe.ingredientNames||[])].some(value=>String(value||'').toLocaleLowerCase('ru').includes(key))));
}
export function sumUnits(items) {
  const result={volume:0,batches:0};
  for (const item of items) result[item.unit==='реакц.'?'batches':'volume']+=Number(item.quantity)||0;
  return result;
}
export function migrateSavedItems(items,recipes,aliases) {
  const byId=new Map(recipes.map(recipe=>[recipe.nameEn,recipe]));
  return items.map(item=>{
    const id=aliases[item.id||item.name] || item.id;
    const recipe=byId.get(id);
    return recipe ? {...item,id:recipe.nameEn,name:recipe.name,unit:recipe.unit} : item;
  });
}
export function temperatureLabel(minimum, maximum) {
  if (minimum != null && maximum != null) return `${minimum}–${maximum}K`;
  if (minimum != null) return `≥ ${minimum}K`;
  if (maximum != null) return `≤ ${maximum}K`;
  return '';
}
