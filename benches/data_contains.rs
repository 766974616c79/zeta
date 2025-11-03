use criterion::{Criterion, criterion_group, criterion_main};
use zeta::Data;

fn criterion_benchmark(c: &mut Criterion) {
    let mut d = Data {
        values: vec![
            String::from(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus suscipit justo a magna dapibus, in porta ipsum auctor. Proin in ornare est. Vivamus vestibulum felis orci, at mattis nisl consequat in. Sed laoreet pretium urna, id volutpat libero vulputate ac. Aliquam tempus ex ac dolor dignissim ornare. Nullam vel nisl leo. Pellentesque sed justo tortor. Donec id quam arcu.",
            ),
            String::from(
                "Donec dui turpis, bibendum id elementum vitae, ullamcorper sit amet mauris. Praesent fringilla mattis risus, et lacinia odio congue sit amet. Nulla sollicitudin vel arcu sit amet viverra. Phasellus ut turpis consequat, feugiat leo eget, interdum ligula. Phasellus purus mi, pulvinar sit amet ultricies at, pulvinar ac elit. Nulla imperdiet erat sed diam fringilla varius. Aliquam condimentum commodo lorem, sit amet bibendum mi. Donec in metus eget ipsum efficitur iaculis. Etiam molestie enim massa, ut malesuada ante laoreet sit amet. Nunc blandit vel dui eget tincidunt. Donec eu hendrerit purus.",
            ),
            String::from(
                "Fusce leo libero, bibendum vitae porta ac, tempor id ligula. Pellentesque commodo quam eget diam vehicula dictum. Nam mattis massa a condimentum ultrices. Cras mi lacus, iaculis non quam vitae, ultrices vehicula dui. Nullam quis ultricies lacus. Ut dignissim ligula leo, sit amet fringilla orci aliquet a. Proin lacus orci, pretium nec nulla at, pharetra bibendum urna. Donec pulvinar, velit quis eleifend, est diam facilisis mauris, eu finibus nulla metus et nisl. Etiam ut libero nec leo vulputate mollis. Pellentesque dictum lectus sit amet lacus ullamcorper condimentum. Morbi accumsan placerat bibendum.", // sagittis removed
            ),
            String::from(
                "Aenean eu finibus nibh, id ullamcorper justo. Nunc et mollis ex, convallis dignissim felis. Proin venenatis dui nec odio lobortis, sed eleifend eros porttitor. Vivamus efficitur vel mi quis congue. In nec euismod erat. Ut vitae velit sit amet sem interdum volutpat non in massa. Vestibulum nunc mauris, venenatis fermentum accumsan sed, lobortis non ligula. Vestibulum metus quam, ullamcorper nec consequat nec, posuere dapibus nulla. Etiam quis congue risus, et fermentum elit. Donec dapibus semper lacus. Vivamus auctor, felis sit amet porttitor cursus, augue nisi rutrum augue, eget suscipit odio ligula non ex. In hac habitasse platea dictumst. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nulla odio nulla, dictum vitae condimentum id, iaculis vitae nulla. Suspendisse potenti.",
            ),
            String::from(
                "Vivamus lobortis, tortor sed pellentesque laoreet, nisl odio vehicula arcu, non pharetra eros nunc et odio. Nullam at ipsum felis. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Nulla et ligula sem. Aliquam elementum tempor metus sed ultricies. Praesent consectetur mi vitae nulla gravida, eget hendrerit felis tempor. Vestibulum congue aliquam dui et semper. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Morbi blandit metus varius iaculis blandit. Nullam gravida mauris enim, vel blandit ex scelerisque non. Quisque ante tellus, elementum vel lectus non, lacinia elementum dolor. Donec lobortis id dolor ut pretium.",
            ),
            String::from(
                "Vestibulum et nisi sit amet ligula blandit gravida. Curabitur convallis eu velit sed aliquam. Mauris id sem eget tellus porttitor facilisis. Aenean rutrum lacinia elit at tempor. Nam dolor felis, sollicitudin id enim id, dapibus fermentum mauris. Interdum et malesuada fames ac ante ipsum primis in faucibus. Proin vitae tincidunt arcu. Sed eleifend lacus ante, vitae imperdiet enim viverra ut. Aenean dapibus velit neque, sit amet condimentum eros placerat ut. Nullam at ipsum blandit, aliquet ligula a, consectetur sapien. Nulla urna eros, ultricies vitae pretium eu, fringilla in nulla. In a scelerisque nunc, sed cursus metus. Nam laoreet sed sem at facilisis.",
            ),
            String::from(
                "Vivamus vel nibh et justo consectetur tristique. Donec in mauris et nisi luctus bibendum at nec urna. Sed rutrum vitae mi in volutpat. Vestibulum ultrices tempor ligula, a tempus justo lacinia eu. Suspendisse eget pretium nisl, id ultrices felis. Praesent nisl turpis, aliquam in varius et, fermentum ac ligula. Maecenas enim justo, hendrerit id eros nec, ultricies suscipit justo. Nunc purus est, facilisis sit amet nisi a, scelerisque tincidunt mauris. Fusce placerat ipsum arcu, ac laoreet nisi vestibulum vehicula. Vivamus malesuada, magna ut vehicula auctor, ligula ex euismod eros, a posuere neque felis vitae eros. Mauris vel arcu a augue sollicitudin scelerisque.",
            ),
            String::from(
                "Aenean aliquam ante nec ligula facilisis, eget scelerisque mauris vestibulum. Nullam pulvinar quam neque, quis porta sapien porttitor at. Proin nec orci ac sapien congue consectetur a sit amet nisl. Maecenas id dui risus. Aenean varius leo nunc, in tincidunt orci condimentum a. Vivamus lorem risus, egestas at lacus non, tristique rutrum lacus. Etiam ac dui dictum, dictum nunc mattis, placerat mauris. Sed varius mi id leo hendrerit ornare. Proin a semper elit. Praesent vitae feugiat lectus. Maecenas erat lectus, euismod vel sapien id, fringilla pellentesque ipsum. Duis at elit euismod, scelerisque elit a, convallis felis.",
            ),
            String::from(
                "Nam ut aliquet augue. Maecenas orci eros, pellentesque quis maximus id, fermentum non ipsum. Quisque sed dictum neque, vitae iaculis nisi. Vivamus auctor sem nec dignissim molestie. Nulla consectetur pharetra metus, nec suscipit sem lobortis vitae. Quisque vel ultricies sapien, vitae interdum ex. Praesent lacinia sem sit amet turpis vulputate efficitur. Aenean purus libero, pulvinar ornare semper in, maximus eget metus. Nullam vel aliquet turpis. Nullam quam velit, imperdiet vitae faucibus vel, sollicitudin nec purus. Phasellus venenatis tellus dui, nec molestie eros accumsan et.",
            ),
            String::from(
                "Praesent consectetur facilisis est quis elementum. Vivamus tincidunt purus ut volutpat scelerisque. Nunc nunc mauris, dictum nec consequat id, molestie vel neque. Mauris suscipit, magna sed dignissim dapibus, erat mi interdum ligula, sit amet consectetur diam mauris quis sapien. Nulla odio mauris, pharetra at magna vel, commodo volutpat leo. Donec vel hendrerit lorem, id porta erat. Morbi ut convallis mi, vel varius felis. In nec felis lacus. Donec vel dui mauris. Phasellus consectetur risus quis viverra auctor. Nam maximus eleifend tellus, ac gravida velit varius et. Sed facilisis ex sit amet metus lobortis, ut venenatis justo scelerisque. Nunc in diam ac magna sagittis tempor vitae eget lectus. Mauris ut odio gravida, feugiat nunc quis, finibus turpis.",
            ),
        ],
        bloom: [0; 128966],
    };

    d.compute();

    c.bench_function("data contains", |b| b.iter(|| d.contains("sagittis")));
    c.bench_function("data bloom contains", |b| {
        b.iter(|| d.bloom_contains("sagittis"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
