pub fn friendly(raw: &str) -> String {
    let text = raw.to_lowercase();
    if text.contains("not enough disk") || text.contains("no space left") {
        return "Espaço insuficiente. Escolha outra pasta de saída ou libere espaço no disco temporário.".into();
    }
    if text.contains("permission denied") || text.contains("acesso negado") {
        return "Sem permissão para gravar. Escolha uma pasta acessível e verifique se o arquivo está aberto em outro programa.".into();
    }
    if text.contains("password") || text.contains("encrypted") {
        return "Arquivo protegido. Confira a senha e o tipo de proteção em Avançado.".into();
    }
    if text.starts_with("engine ") {
        return "O motor não conseguiu processar este arquivo. Tente outro formato ou uma configuração mais simples. O diagnóstico está em Detalhes.".into();
    }
    raw.into()
}
