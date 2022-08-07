var §%{name} = itemRegistry.createTemplate("§%{name}", 1);
§%{name}.withAttackHandler((function(player, angle, power) {
    var result = gameServer.raytrace(player.location(), angle, §%{distance}, (function(ent){
        return ent.getId() != player.getId();
    }));
    if(result.entity != null)
        result.entity.damage(-§%{damage}, EDamageType.BULLET);
    gameServer.spawnParticle("gunshot", player.location(), 5).addNumber("length", result.distance).addNumber("angle", angle);
}));
§%{name}.addRenderState("default", "assets/items/§%{name}.png");
§%{name}.register();